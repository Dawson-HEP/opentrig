module main(
    // PLL
    input wire a_clk,       // 10 MHz
    input wire b_clk,       // 40 MHz
    output wire pll_clk,
    output wire pll_lock,

    // SPI
    input wire spi_clk,
    input wire spi_cs,
    input wire spi_si,
    output reg spi_so,

    // Trigger
    input wire trig_in,
    input wire veto_in,
    output wire trig_out,
    output wire veto_out,

    // Status
    input wire global_reset,
    output wire mem_swap_interrupt
);

    wire clk;  // Output of PLL
    assign pll_clk = clk;

    // PLL instance
    pll pll_inst (
        .clock_in(a_clk),
        .clock_out(clk),
        .locked(pll_lock)
    );

    // === RAM signals ===
    wire [7:0] RDATA;
    reg [10:0] RADDR = 0, WADDR = 0;
    reg [7:0] WDATA = 0;
    wire [7:0] MASK = 8'hFF;

    reg RCLKE = 1, RE = 0;
    reg WCLKE = 1, WE = 0;

    reg RCLK = 0, WCLK = 0;

    // === RAM instance ===
    SB_RAM40_4K ram40_4kinst_physical (
        .RDATA(RDATA),
        .RADDR(RADDR),
        .RCLK(RCLK),
        .RCLKE(RCLKE),
        .RE(RE),
        .WADDR(WADDR),
        .WCLK(WCLK),
        .WCLKE(WCLKE),
        .WE(WE),
        .MASK(MASK),
        .WDATA(WDATA)
    );

    // SPI FSM
    reg [2:0] bit_cnt = 0;
    reg [7:0] shift_reg = 0;
    reg [7:0] address_reg = 0;
    reg [1:0] spi_state = 0;
    reg [7:0] read_buffer = 0;

    // States
    localparam IDLE = 2'd0;
    localparam ADDR = 2'd1;
    localparam WRITE = 2'd2;
    localparam READ = 2'd3;

    // Synchronize spi_clk domain (assumes spi_clk is slow and clean)
    always @(posedge spi_clk or posedge spi_cs) begin
        if (spi_cs) begin
            bit_cnt <= 0;
            spi_state <= ADDR;
            spi_so <= 0;
        end else begin
            shift_reg <= {shift_reg[6:0], spi_si};
            bit_cnt <= bit_cnt + 1;

            if (bit_cnt == 7) begin
                case (spi_state)
                    ADDR: begin
                        address_reg <= {shift_reg[6:0], spi_si};
                        RADDR <= {3'b000, {shift_reg[6:0], spi_si}}; // 8-bit address
                        spi_state <= READ;
                        RE <= 1;
                        RCLK <= ~RCLK;  // Toggle read clock
                        #1;  // Delay for read to complete
                        read_buffer <= RDATA;
                    end
                    READ: begin
                        // Send back read byte
                        spi_so <= shift_reg[7]; // MSB first
                        read_buffer <= {read_buffer[6:0], 1'b0};
                    end
                    WRITE: begin
                        WADDR <= {3'b000, address_reg};
                        WDATA <= {shift_reg[6:0], spi_si};
                        WE <= 1;
                        WCLK <= ~WCLK;  // Toggle write clock
                        spi_state <= IDLE;
                    end
                    default: spi_state <= IDLE;
                endcase
            end else if (spi_state == READ) begin
                // Output bits from read_buffer
                spi_so <= read_buffer[7];
                read_buffer <= {read_buffer[6:0], 1'b0};
            end
        end
    end

    assign trig_out = trig_in;
    assign veto_out = veto_in;
    assign mem_swap_interrupt = 0;

endmodule