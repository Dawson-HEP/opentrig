module spi(
    input wire sampling_clk,
    input wire clk_async,
    input wire cs_async,
    output reg so,
    input wire [15:0] data,
);
    sync sync_spi_clk (
        .async(clk_async),
        .clk(sampling_clk),
        .falling(clk_falling)
    );

    sync sync_spi_cs (
        .async(cs_async),
        .clk(sampling_clk),
        .falling(cs_falling)
    );

    reg [15:0] sr;
    reg [5:0] count;
    reg done;

    always @(posedge sampling_clk) begin
        if (cs_falling) begin
            sr <= data;
            count <= 6'b0;
            done <= 1'b0;
        end else if (clk_falling && !done) begin
            so <= sr[15];
            sr <= {sr[14:0], 1'b0};
            count <= count + 1;

            if (count == 16) begin
                done <= 1'b1;
            end
        end
    end
endmodule