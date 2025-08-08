module latch (
    input wire sampling_clk,
    input wire sample_interrupt,
    input wire [23:0] inputs_async,
    output reg [23:0] out
);
    // store n cycles worth of data
    localparam latch_length = 16;
    localparam latch_length_minus_one = latch_length - 1;

    reg [latch_length:0] sr [23:0];
    reg [23:0] current;

    integer i;
    always @(posedge sampling_clk) begin
        for (i = 0; i < 24; i = i + 1) begin
            sr[i] <= {sr[i][latch_length_minus_one:0], inputs_async[i]};
            current[i] <= |sr[i];
        end

        if (sample_interrupt) begin
            out <= current;
        end
    end
endmodule
