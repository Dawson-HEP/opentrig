module trigger(
    input wire sampling_clk,

    input wire trig_in_async,
    input wire trig_id_async,
    input wire clk_in_async,

    output reg interrupt,
    output reg [15:0] trigger_id,
);
    sync sync_trig_in (
        .async(trig_in_async),
        .clk(sampling_clk),
        .sync(trig_in_sync)
    );
    sync sync_trig_id (
        .async(trig_id_async),
        .clk(sampling_clk),
        .sync(trig_id_sync)
    );
    sync sync_clk_in (
        .async(clk_in_async),
        .clk(sampling_clk),
        .falling(clk_in_falling)
    );

    reg [3:0] count;
    reg capturing;
    always @(posedge sampling_clk) begin
        if (trig_in_sync) begin
            trigger_id <= 16'b0;
            count <= 4'b0;
            capturing <= 1'b1;
        end else begin
            interrupt <= 1'b0;
        end
        if (capturing & clk_in_falling) begin
            trigger_id <= {trigger_id[14:0], trig_id_sync};
            count <= count + 1;
            if (count == 15) begin
                capturing <= 1'b0;
                interrupt <= 1'b1;
            end
        end
    end

endmodule