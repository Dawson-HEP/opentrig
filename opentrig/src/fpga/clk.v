module clk_ref (
    input wire sampling_clk,

    input wire clk_in_rising,
    input wire reset_falling,

    output reg [47:0] ref
);
    reg [23:0] low, high;
    reg incr_high;

    always @(posedge sampling_clk) begin
        if (reset_falling) begin
            low <= 0;
            high <= 0;
            incr_high <= 0;
        end else begin
            if (clk_in_rising) begin
                ref <= {high, low};

                if (low == 24'hFF_FFFF) begin
                    incr_high <= 1;
                end
                low <= low + 1;
            end else if (incr_high) begin
                high <= high + 1;
                incr_high <= 0;
            end
        end
    end
endmodule
