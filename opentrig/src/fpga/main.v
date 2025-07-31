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
    output reg trig_out,
    output reg veto_out,

    // Status
    // input wire global_reset,
    // output wire mem_swap_interrupt

    // MCU management
    input wire reset,               // active low

    // Inputs
    input wire [23:0] c_input,      // active high, comparator output

    // Debug connectors
    output reg [23:0] debug,
    output wire [7:0] led,
);
    pll pll_inst (
        .clock_in(mcu_clk),
        .clock_out(pll_clk),
        .locked(pll_lock)
    );

    assign led[7:0] = 8'b10101010;

    /// RESET
    reg reset_sync_0, reset_sync_1;                 // 2-FF input sync
    wire reset_synchronous;                         // active-high internal reset line
    assign reset_synchronous = ~reset_sync_1;
    always @(posedge pll_clk) begin
        reset_sync_0 <= reset;
        reset_sync_1 <= reset_sync_0;
    end

    // TRIG-IN
    reg trig_in_sync_0, trig_in_sync_1;             // 2-FF input sync
    wire trig_in_synchronous;                       // active-high internal reset line
    assign trig_in_synchronous = trig_in_sync_1;
    always @(posedge pll_clk) begin
        trig_in_sync_0 <= trig_in;
        trig_in_sync_1 <= trig_in_sync_0;
    end

    /// TRIGGER
    // 2-FF input synchronizers
    reg [23:0] c_input_sync_0, c_input_sync_1, c_input_sync_2, commit_stage;
    reg [4:0] commit_timeout;

    // if any events are captured into the commit-stage,
    // then commit_pending is active high;
    wire commit_pending;
    assign commit_pending = |commit_stage;

    // commit timeout
    // if commit_stage has been active for n clock cycles -> reset
    // 24clk -> 200ns
    localparam [5:0] timeout = 24;
    // precompute conditions for three cycles of trig_out -> one clock cycle at 40Mhz
    localparam [5:0] timeout_plus_1 = timeout + 1;
    localparam [5:0] timeout_plus_2 = timeout + 2;
    localparam [5:0] timeout_plus_3 = timeout + 3;

    always @(posedge pll_clk) begin
        if (reset_synchronous) begin
        // reset -> reset all registers
            c_input_sync_0 <= 24'b0;
            c_input_sync_1 <= 24'b0;
            c_input_sync_2 <= 24'b0;

            commit_stage <= 24'b0;
            commit_timeout <= 5'b0;

            trig_out <= 1'b0;
            veto_out <= 1'b0;

            // clear debug
            // debug <= 24'b0;
        end else begin
        // reset -> normal operation: await trigger
            // 2-FF synchronizer for c_input
            c_input_sync_0 <= c_input;
            c_input_sync_1 <= c_input_sync_0;
            c_input_sync_2 <= c_input_sync_1;

            // accumulate one-shot synchronized events
            // into the commit-stage
            commit_stage <= commit_stage | (c_input_sync_1 & ~c_input_sync_2);

            if (commit_timeout == timeout || trig_in_synchronous) begin
                // TODO: commit commit stage
                // debug <= commit_stage;
                commit_timeout <= commit_timeout + 1;
                trig_out <= 1'b1;
            end else if (
                commit_timeout == timeout_plus_1 ||
                commit_timeout == timeout_plus_2
            ) begin
                commit_timeout <= commit_timeout + 1;
                trig_out <= 1'b1;
            end else if (
                commit_timeout == timeout_plus_3
            ) begin
                // finish the trigger_out cycle
                // and reset + purge commit_stage
                commit_stage <= 24'b0;
                commit_timeout <= 5'b0;
                trig_out <= 1'b0;
            end else begin
                // reset trigger_output
                trig_out <= 1'b0;

                // if pending captures
                if (commit_pending) begin
                    // currently busy
                    veto_out <= 1'b1;

                    // increment timeout
                    commit_timeout <= commit_timeout + 1;
                end else begin
                    // not busy
                    veto_out <= 1'b0;
                end
            end
        end
    end

endmodule
