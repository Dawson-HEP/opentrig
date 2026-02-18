/**
 * EXTERNAL TRIGGER
 *
 * External trigger event handler
 * with DESY trigger-ID parsing
 */
module trigger(
    input wire sampling_clk,

    input wire trig_in_async,
    input wire trig_id_async,
    input wire clk_in_async,
    input wire reset_async,

    output wire sample_interrupt,

    output reg interrupt,
    output reg [15:0] trigger_id,
    output reg [63:0] trigger_cycle,
);
    // rising edge of trig_in_async
    // triggers the sample for the whole system
    // -> sample_interrupt
    sync sync_trig_in (
        .async(trig_in_async),
        .clk(sampling_clk),
        .sync(trig_in_sync),
        .rising(sample_interrupt)
    );
    sync sync_trig_id (
        .async(trig_id_async),
        .clk(sampling_clk),
        .sync(trig_id_sync)
    );
    sync sync_clk_in (
        .async(clk_in_async),
        .clk(sampling_clk),
        .falling(clk_in_falling),
        .rising(clk_in_rising)
    );
    sync sync_reset (
        .async(reset_async),
        .clk(sampling_clk),
        .falling(reset_falling)
    );

    // current clock cycle counting
    reg [63:0] cycle;
    clk_ref clk_ref_inst (
        .sampling_clk(sampling_clk),
        .clk_in_rising(clk_in_rising),
        .reset_falling(reset_falling),
        .ref(cycle)
    );

    // trigger-ID bit count
    reg [3:0] count;
    reg capturing;
    always @(posedge sampling_clk) begin
        if (trig_in_sync) begin
            // trigger active-high
            trigger_id <= 16'b0;
            count <= 4'b0;
            capturing <= 1'b1;
        end else begin
            interrupt <= 1'b0;
        end
        if (capturing & clk_in_falling) begin
            trigger_cycle <= cycle; // record trig_in falling cycle;

            // shift trigger-ID into SR
            trigger_id <= {trigger_id[14:0], trig_id_sync};
            count <= count + 1;
            if (count == 15) begin
                capturing <= 1'b0;
                interrupt <= 1'b1;
                // trigger-ID is ready
                // -> call MCU interrupt for SPI readout
            end
        end
    end
endmodule

/**
 * INTERNAL TRIGGER
 *
 * Internal trigger event handler
 * with delay
 */
module trigger_internal (
    input wire [23:0] inputs_async,
    input wire sampling_clk,
    output reg trigger,
    // output reg trigger_long,
);  
    // rising edge detection for all inputs    
    reg [23:0] sync_0, sync_1;
    wire [23:0] rising = sync_0 & ~sync_1;

    // any inputs are rising
    wire any_rising = |rising;

    // await-incoming-events counter
    reg counting;
    reg [4:0] count;

    // internal trigger occurs n cycles after input rise.
    // ensure latch.v has a long enough buffer.
    localparam trig_in_rise_clk = 31;
    localparam trig_in_fall_clk = trig_in_rise_clk + 3;

    always @(posedge sampling_clk) begin
        // edge detection on inputs
        sync_0 <= inputs_async;
        sync_1 <= sync_0;

        if (counting) begin
            count <= count + 1;

            if (count == trig_in_rise_clk) begin
                // active internal trigger
                trigger <= 1;
            end else if (count == trig_in_fall_clk) begin
                // reset trigger
                trigger <= 0;
                counting <= 0;
            end
        end else begin
            // any channel is activated
            // -> start awaiting on the trigger clk count
            if (any_rising) begin
                count <= 0;
                counting <= 1;
            end
        end
    end
endmodule
