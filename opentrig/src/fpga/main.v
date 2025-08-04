module main(
    // PLL
    input wire mcu_clk,       // 10 MHz
    input wire ext_clk,       // 40 MHz
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
    input wire trig_id,
    output reg trig_out,
    output reg veto_out,

    // Status
    // input wire global_reset,
    // output wire mem_swap_interrupt

    

    // MCU management
    input wire interrupt,           // active low
    input wire reset,               // active low

    // Inputs
    input wire [23:0] c_input,      // active high, comparator output

    // Debug connectors
    input wire [23:0] debug,
    output wire [7:0] led,

    // Auxiliary outputs for debug
    output wire [9:0] aux_out,
);
    wire clk_in = ext_clk;

    // auxiliary debug ports
    assign aux_out[0] = spi_clk;
    assign aux_out[1] = spi_cs;
    assign aux_out[2] = spi_so;
    assign aux_out[3] = spi_si;
    assign aux_out[4] = trig_out;
    assign aux_out[5] = veto_out;
    assign aux_out[6] = pll_clk;
    assign aux_out[7] = clk_in;
    assign aux_out[8] = trig_in;
    assign aux_out[9] = trig_id;

    pll pll_inst (
        .clock_in(clk_in),
        .clock_out(pll_clk),
        .locked(pll_lock)
    );

    reg [23:0] clk_counter = 0;
    always @(posedge clk_in) begin
        clk_counter <= clk_counter + 1;
    end

    // leds map
    // (4) PLL lock     (0) counter bit 20
    // (5)              (1) counter bit 21
    // (6)              (2) counter bit 22
    // (7)              (3) counter bit 23
    assign led[3:0] = clk_counter[23:20];
    assign led[4] = pll_lock;
    assign led[5] = trig_in;
    assign led[7:6] = 2'b0;

    // wire [15:0] fifo_rdata;
    // reg  [7:0] fifo_waddr, fifo_raddr;
    // reg  [15:0] fifo_wdata;
    // reg        fifo_we, fifo_re;

    // SB_RAM40_4K ram40_fifo (
    //     .RDATA(fifo_rdata),
    //     .RADDR(fifo_raddr),
    //     .RCLK(spi_clk),
    //     .RCLKE(1'b1),
    //     .RE(fifo_re),

    //     .WADDR(fifo_waddr),
    //     .WDATA(fifo_wdata),
    //     .WCLK(clk_in),
    //     .WCLKE(1'b1),
    //     .WE(fifo_we),
    //     .MASK(16'h0000)  // no masking
    // );

    // defparam ram40_fifo.READ_MODE = 0;
    // defparam ram40_fifo.WRITE_MODE = 0;

    // TRIG-IN
    // reg trig_in_sync_0, trig_in_sync_1;             // 2-FF input sync
    // wire trig_in_synchronous = trig_in_sync_1;
    // reg trig_id_sync_0, trig_id_sync_1;
    // wire trig_id_synchronous = trig_id_sync_1;
    // reg clk_in_sync_0, clk_in_sync_1;
    // wire clk_in_falling = ~clk_in_sync_0 & clk_in_sync_1;
    // always @(posedge pll_clk) begin
    //     trig_in_sync_0 <= trig_in;
    //     trig_in_sync_1 <= trig_in_sync_0;
    //     trig_id_sync_0 <= trig_id;
    //     trig_id_sync_1 <= trig_id_sync_0;
    //     clk_in_sync_0 <= clk_in;
    //     clk_in_sync_1 <= clk_in_sync_0;
    // end

    // TRIGGER-ID
    // reg [15:0] trigger_id;
    // reg [4:0] bit_count;
    // reg capturing;
    // always @(posedge pll_clk) begin
    //     if (trig_in_synchronous) begin
    //         trigger_id <= 16'b0;
    //         bit_count <= 5'b0;
    //         capturing <= 1'b1;
    //     end else begin
    //         interrupt <= 1'b0;
    //     end
    //     if (capturing & clk_in_falling) begin
    //         trigger_id <= {trigger_id[14:0], trig_id_synchronous};
    //         bit_count <= bit_count + 1;
    //         if (bit_count == 15) begin
    //             capturing <= 1'b0;
    //             interrupt <= 1'b1;
    //         end
    //     end
    // end

endmodule
