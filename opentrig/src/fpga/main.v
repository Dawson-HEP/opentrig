/**
 * MAIN
 *
 * Opentrig DAQ
 * FPGA entrypoint
 */
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
    output wire [7:0] led,

    // Auxiliary outputs for debug
    output wire [9:0] aux_out,
);
    wire clk_in = ext_clk;

    // auxiliary debug ports
    assign aux_out[0] = spi_clk;
    assign aux_out[1] = spi_cs;
    assign aux_out[2] = spi_so;
    assign aux_out[3] = c_input[0];
    assign aux_out[4] = sample_interrupt;
    assign aux_out[5] = veto_out;
    assign aux_out[6] = data_reg[38];
    assign aux_out[7] = clk_in;
    assign aux_out[8] = trig_in;
    assign aux_out[9] = trig_id;

    pll pll_inst (
        .clock_in(clk_in),
        .clock_out(pll_clk),
        .locked(pll_lock)
    );

    // small binary counter on the LEDs
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
    assign led[5] = veto_out;
    assign led[7:6] = 2'b0;

    // MAIN DATA REG
    reg [127:0] data_reg;
    assign data_reg[127:120] = 8'h7E;
    assign data_reg[7:0] = 8'h7D;

    // MAPPING
    //
    // 128-120  0x7E start byte
    // 119      0x00 MSB TRIG-ID
    //     104  0x00 LSB TRIG-ID
    // 103      0x00 MSB counter
    //      88  0x00
    //  87      0x00
    //          0x00
    //          0x00
    //          0x00
    //          0x00
    //      40  0x00 LSB counter
    //  39      0x00 EXTRA
    //      39      VETO
    //      38      INTERNAL TRIGGER
    //  31      0x00 MSB data
    //          0x00 
    //       8  0x00 LSB data
    //          0x7D end byte

    // // INTERNAL TRIGGER
    // wire sample_interrupt;      
    // // synchronous, active-high sampling event aligned to pll_clk
    // reg trig_in_internal;
    // // internal trigger
    // trigger_internal trigger_internal_inst (
    //     .inputs_async(c_input),
    //     .sampling_clk(pll_clk),
    //     .trigger(trig_in_internal),
    // );
    // // set internal trigger bit
    // always @(posedge pll_clk) begin
    //     if (sample_interrupt && trig_in) begin
    //         data_reg[38] <= 0; // external trigger on sample
    //     end else if (sample_interrupt && trig_in_internal) begin
    //         data_reg[38] <= 1; // internal trigger on sample
    //     end
    // end

    // combine internal and external trigger
    // trig_in -> higher priority (veto_out will be valid before trig_in_internal)
    // wire trig_in_combined = (trig_in_internal & ~veto_out) || trig_in;

    // // EXTERNAL TRIGGER
    // trigger trigger_inst (
    //     .sampling_clk(pll_clk),
    //     .trig_in_async(trig_in_combined),
    //     .trig_id_async(trig_id),
    //     .clk_in_async(clk_in),
    //     .reset_async(reset),
    //     .sample_interrupt(sample_interrupt),
    //     .interrupt(interrupt),
    //     .trigger_id(data_reg[119:104]),
    //     .trigger_cycle(data_reg[87:40])
    // );

    coincidence_trigger coincidence_trigger_inst(
        .inputs_async(data_reg[9:8]),
        .clk(pll_clk),
        .out(trig_out)
    )

    // INPUT LATCHES
    latch latch_inst(
        .sampling_clk(pll_clk),
        .sample_interrupt(sample_interrupt),
        .inputs_async(c_input),
        .out(data_reg[31:8])
    );

    // SPI INTERFACE
    wire sample_done;
    // spi transfer completion flag, active-high
    spi spi_inst (
        .sampling_clk(pll_clk),
        .clk_async(spi_clk),
        .cs_async(spi_cs),
        .so(spi_so),
        .data(data_reg),
        .sample_done(sample_done)
    );

    // WATCHDOG
    reg clear_force;
    // spi read timeout flag, active-high
    watchdog watchdog_inst (
        .sampling_clk(pll_clk),
        .cs_async(spi_cs),
        .reset_async(reset),
        .sample_interrupt(sample_interrupt),
        .clear_force(clear_force)
    );
    // either spi times out, or is read by the MCU -> move onto next sample
    wire ready_for_next_sample = clear_force || sample_done;

    // VETO SIGNALS
    sync sync_veto (
        .async(veto_in),
        .clk(pll_clk),
        .sync(veto_in_sync)
    );
    always @(posedge pll_clk) begin
        if (sample_interrupt) begin
           veto_out <= 1;  // DAQ is busy in acquisition
        end else if (ready_for_next_sample) begin
            veto_out <= 0; // DAQ ready for next
        end

        data_reg[39] <= veto_in_sync; // Record if external veto is active
    end

endmodule
