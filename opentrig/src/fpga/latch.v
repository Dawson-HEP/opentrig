/**
 * LATCH
 *
 * Fast input latching, and longitudinal,
 * synchronous sampling with respect to clock
 */
`include "settings/settings.v"

module latch (
    input wire sampling_clk,
    input wire sample_interrupt,
    input wire [23:0] inputs_async,
    output reg [23:0] out
);
    // store n cycles worth of data
    localparam latch_length = 32;
    localparam latch_length_minus_one = latch_length - 1;

    // longitudinal buffer over latch_length cycles
    // shift register which gets shifted per sampling_clk
    reg [latch_length:0] sr [23:0];

    // OR-gate on each shift register
    // denotes whether a channel has been active at least once
    // over the last latch_length cycles
    reg [23:0] current;

    integer i;
    always @(posedge sampling_clk) begin
        for (i = 0; i < 24; i = i + 1) begin
            // shift new channel levels
            sr[i] <= {sr[i][latch_length_minus_one:0], inputs_async[i]};

            // OR-gate reduction of every SR -> longitudinal trigger
            current[i] <= |sr[i];
        end

        // if sample event is called, shift the active channels
        // into out
        if (sample_interrupt) begin
            out <= current & `INPUT_MASK;
        end
    end
endmodule
