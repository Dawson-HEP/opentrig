/**
 * CLK
 *
 * Input clock cycle counting
 * with 48-bit counter
 */
module clk_ref (
    input wire sampling_clk,

    input wire clk_in_rising,
    input wire reset_falling,

    output reg [47:0] ref
);
    // store LSB and MSB in two registers
    // executes 48-bit addition over two cycles (24-bit each)
    // to maintain fast FPGA clock speed
    reg [23:0] low, high;
    // if MSB needs to be involved during a carry
    reg incr_high;

    // assumption that sampling_clk (120Mhz) runs faster
    // than clk_in_rising (40Mhz)
    // this is needed to do two-cycle addition
    always @(posedge sampling_clk) begin
        if (reset_falling) begin
            low <= 0;
            high <= 0;
            incr_high <= 0;
        end else begin
            if (clk_in_rising) begin
                // current clock cycle
                ref <= {high, low};

                // LSB requires carry to MSB
                if (low == 24'hFF_FFFF) begin
                    incr_high <= 1;
                end

                // increment LSB
                low <= low + 1;
            end else if (incr_high) begin
                // execute carry of MSB
                high <= high + 1;
                incr_high <= 0;
            end
        end
    end
endmodule
