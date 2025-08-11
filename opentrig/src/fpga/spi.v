/**
 * SPI
 *
 * Serial peripheral interface
 * for 128-bit data readout from FPGA to MCU
 */
module spi(
    input wire sampling_clk,
    input wire clk_async,
    input wire cs_async,
    output wire sample_done,
    output reg so,
    input wire [127:0] data,
);
    sync sync_spi_clk (
        .async(clk_async),
        .clk(sampling_clk),
        .falling(clk_falling)
    );

    sync sync_spi_cs (
        .async(cs_async),
        .clk(sampling_clk),
        .falling(cs_falling),
        .rising(sample_done)
    );

    // spi 128bit shift register
    reg [127:0] sr;
    // shifted bits count
    reg [7:0] count;
    // transfer complete, active-high
    reg done;

    // CPOL=1, CPHA=1, CS active-low
    always @(posedge sampling_clk) begin
        if (cs_falling) begin
            // reset spi interface on CS falling
            sr <= data;
            count <= 8'b0;
            done <= 1'b0;
        end else if (clk_falling && !done) begin
            // start shifting out data on falling-edge of clk
            so <= sr[127];
            sr <= {sr[126:0], 1'b0};
            count <= count + 1;

            // shifting complete
            if (count == 128) begin
                done <= 1'b1;
            end
        end
    end
endmodule
