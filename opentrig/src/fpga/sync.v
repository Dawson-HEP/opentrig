module sync(
    input wire async,
    input wire clk,
    output wire sync,
    output wire rising,
    output wire falling,
);
    reg sync_0, sync_1;

    assign sync = sync_1;
    assign rising = sync_0 & ~sync_1;
    assign falling = ~sync_0 & sync_1;

    always @(posedge clk) begin
        sync_0 <= async;
        sync_1 <= sync_0;
    end
endmodule
