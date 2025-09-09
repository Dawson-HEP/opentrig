/**
 * WATCHDOG
 *
 * Watchdog for MCU hanging
 * detection, and FPGA auto-reset
 */
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

    // watchdog timer
    reg [13:0] watchdog_counter;
    reg counting;

    // number of clock cycles the watchdog counts
    // until a forceful reset is called
    localparam watchdog_await_cycles = 5000;
    localparam watchdog_await_cycles_next = watchdog_await_cycles + 1;

    always @(posedge sampling_clk) begin
        if (watchdog_reset) begin
            // asynchronous reset from MCU
            counting <= 0;
            watchdog_counter <= 0;
            clear_force <= 0;
        end else if (sample_interrupt) begin
            // incoming sample -> awaiting MCU read from SPI
            // activate watchdog to reset if ever MCU misses interrupt
            counting <= 1;
            watchdog_counter <= 0;
        end else if (counting) begin
            // increment watchdog counter
            watchdog_counter <= watchdog_counter + 1;
            if (watchdog_counter == watchdog_await_cycles) begin
                // force clear interrrupt
                clear_force <= 1;
            end else if (watchdog_counter == watchdog_await_cycles_next) begin
                // reset watchdog
                clear_force <= 0;
                counting <= 0;
            end else if (cs_falling) begin
                // SPI read occurring resets watchdog
                clear_force <= 0;
                counting <= 0; 
            end
        end
    end
endmodule
