import {CardStats} from "@/components/ui/dashboard/card-stats";

import { MapPin, Waypoints, Navigation, ArrowUpRight } from "lucide-react";
import {Card, CardDescription, CardHeader, CardTitle} from "@/components/ui/card";
import {Button} from "@/components/ui/button";

import Link from "next/link";

const STEPS = [
    {
        title: "Harvesting data",
        cards: []
    },
    {
        title: "Importing data",
        cards: []
    },
    {
        title: "Clustering",
        cards: [
            { title: "Clustered stops", description: "Explore the calculated clustering", link: "/dashboard/map" }
        ]
    },
    {
        title: "Pre-Calculate connections",
        cards: []
    }
]

export default function HomePage() {
    return (
        <div className="grid items-start gap-4 p-4 sm:p-6 md:gap-8">
            <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
                <CardStats
                    title="Stops" subtitle="Stop places accross datasets" value="10,002"
                    icon={<MapPin />} />
                <CardStats
                    title="Lines" subtitle="Lines accross datasets" value="4,202"
                    icon={<Waypoints />} />
                <CardStats
                    title="Trips" subtitle="Trips places accross datasets" value="34,502"
                    icon={<Navigation />} />
            </div>
            <div className="flex flex-col gap-4 md:gap-8">
                {STEPS.map((step) => (
                    <>
                        <h2 className="text-lg font-semibold leading-none tracking-tight">{step.title}</h2>
                        {step.cards.map((card) => (
                            <Link href={card.link} key={card.link}>
                                <Card className="flex flex-row items-center">
                                    <CardHeader className="bg-muted/50">
                                        <CardTitle>{card.title}</CardTitle>
                                        <CardDescription>{card.description}</CardDescription>
                                        <Button variant="link" className="gap-1">
                                            Learn more
                                            <ArrowUpRight className="w-4 h-4" />
                                        </Button>
                                    </CardHeader>
                                </Card>
                            </Link>
                        ))}
                    </>
                ))}
            </div>
        </div>
    );
}
