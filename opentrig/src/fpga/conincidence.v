/*
* COINCIDENCE TRIGGER
*  
* Triggers a pulse of adjustable
* duration upon coincidence of
* the two or more inputs. 
*/
`include "settings/settings.v"

module trigger_coincidence (
    input wire [23:0] inputs_async,
    input wire clk,
    output reg trigger
);
    reg [23:0] sync_0, sync_1;
    wire [23:0] rising = sync_0 & ~sync_1;
    reg counting;
    reg [7:0] count;

    always @(posedge clk) begin
        sync_0 <= inputs_async;
        sync_1 <= sync_0;

            if (counting) begin
                count <= count + 1;

                if (count >= `COINCIDENCE_OUT_N_CYCLES - 1) begin
                    counting <= 0;
                    trigger <= 0;

                    count <= 0;
                end
            end else if ((rising & `COINCIDENCE_MASK) == `COINCIDENCE_MASK) begin
                counting <= 1;
                trigger <= 1;
            end

        // Works but we don't know why. Scary!!!
        // if (counting) begin
        //     count <= count + 1;

        //     if (count == `COINCIDENCE_OUT_N_CYCLES) begin
        //         counting <= 0;
        //         trigger <= 0;
        //         count <= 0;
        //     end
        // end 
        // else if (|(rising)) begin  // We have no clue why this works. But it does.            counting <= 1;
        //  s   trigger <= 1;
        // end
    end

endmodule
