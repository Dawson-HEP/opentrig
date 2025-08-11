module watchdog (
    input wire sampling_clk,
    input wire cs_async,
    input wire reset_async,

    input wire sample_interrupt,

    output reg clear_force,
);
    sync sync_cs (
        .async(cs_async),
        .clk(sampling_clk),
        .falling(cs_falling)
    );
    sync sync_reset (
        .async(reset_async),
        .clk(sampling_clk),
        .falling(watchdog_reset)
    );

    reg [11:0] watchdog_counter;
    reg counting;

    localparam watchdog_await_cycles = 2000;
    localparam watchdog_await_cycles_next = watchdog_await_cycles + 1;

    always @(posedge sampling_clk) begin
        if (watchdog_reset) begin
            counting <= 0;
            watchdog_counter <= 0;
            clear_force <= 0;
        end else if (sample_interrupt) begin
            counting <= 1;
            watchdog_counter <= 0;
        end else if (counting) begin
            watchdog_counter <= watchdog_counter + 1;
            if (watchdog_counter == watchdog_await_cycles) begin
                clear_force <= 1;
            end else if (watchdog_counter == watchdog_await_cycles_next) begin
                clear_force <= 0;
                counting <= 0;
            end else if (cs_falling) begin
               clear_force <= 0;
               counting <= 0; 
            end
        end
    end
endmodule
