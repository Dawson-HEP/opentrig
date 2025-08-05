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
    output wire interrupt,           // active low
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
    assign aux_out[4] = rclk;
    assign aux_out[5] = interrupt;
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

    // MAIN DATA REG
    reg [127:0] data_reg;
    assign data_reg[127:120] = 8'h7E;
    assign data_reg[7:0] = 8'h7D;
    
    // 128-120  0x7E start byte
    // 119      0x00 MSB TRIG-ID
    //     104  0x00 LSB TRIG-ID
    // 103      0x00 MSB counter
    //          0x00
    //          0x00
    //          0x00
    //          0x00
    //          0x00
    //          0x00
    //      40  0x00 LSB counter
    //  39      0x00 MSB data
    //          0x00
    //      16  0x00 LSB data
    //          0x00 ZERO
    //          0x7D end byte

    // TRIGGER
    trigger trigger_inst (
        .sampling_clk(pll_clk),
        .trig_in_async(trig_in),
        .trig_id_async(trig_id),
        .clk_in_async(clk_in),
        .reset_async(reset),
        .interrupt(interrupt),
        .trigger_id(data_reg[119:104]),
        .trigger_cycle(data_reg[103:40])
    );

    // SPI interface
    spi spi_inst (
        .sampling_clk(pll_clk),
        .clk_async(spi_clk),
        .cs_async(spi_cs),
        .so(spi_so),
        .data(data_reg)
    );

endmodule
