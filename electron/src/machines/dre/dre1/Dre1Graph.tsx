import { Page } from "@/components/Page";
import { dre1SerialRoute } from "@/routes/routes";
import React from "react";

export function Dre1GraphsPage() {
    const { serial } = dre1SerialRoute.useParams();
    return <Page>Dre 1 Graph</Page>;
}
