module latch (
    input wire sampling_clk,
    input wire sample_interrupt,
    input wire [23:0] inputs_async,
    output reg [23:0] out
);
    reg [11:0] sr [23:0];
    reg [23:0] current;

    integer i;
    always @(posedge sampling_clk) begin
        for (i = 0; i < 24; i = i + 1) begin
            sr[i] <= {sr[i][10:0], inputs_async[i]};
            current[i] <= |sr[i];
        end

        if (sample_interrupt) begin
            out <= current;
        end
    end
endmodule
