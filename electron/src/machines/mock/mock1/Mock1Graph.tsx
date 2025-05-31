import { Page } from "@/components/Page";
import { BigGraph } from "@/helpers/BigGraph";
import React from "react";
import { useMock1 } from "./useMock";

export function Mock1GraphPage() {
    const {
        sineWave,
    } = useMock1();
    return <Page>
        <div style={{ width: '1000px', height: '400px' }}>
            <BigGraph
                newData={sineWave}
                threshold1={75}
                threshold2={50}
                target={60}
                unit="mm"
            />
        </div>
    </Page>;
}


