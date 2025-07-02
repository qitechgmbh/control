import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React from "react";

export function Buffer1ControlPage() {
    return(
        <Page>
            <ControlGrid>
                <Label label="Button Test">
                    <TouchButton
                        variant="outline"
                        icon="lu:ArrowUpToLine"
                    >
                    </TouchButton>
              </Label>

            </ControlGrid>
        </Page>
    );
}