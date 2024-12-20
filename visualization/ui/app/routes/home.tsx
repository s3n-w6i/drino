import type {Route} from "./+types/home";
import {ArrowUpRight, MapPin, Navigation, Waypoints} from "lucide-react";
import {CardStats} from "~/components/ui/dashboard/card-stats";
import {NavLink} from "react-router";
import {Card, CardDescription, CardHeader, CardTitle} from "~/components/ui/card";
import {Button} from "~/components/ui/button";
import {useEffect, useState} from "react";
import {LoadingSpinner} from "~/components/ui/spinner";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Home"},
    ];
}

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
            {title: "Clustered stops", description: "Explore the calculated clustering", link: "map"}
        ]
    },
    {
        title: "Pre-Calculate connections within clusters",
        cards: []
    },
    {
        title: "Pre-Calculate connections between clusters",
        cards: []
    }
]

interface Stats {
    num_stops: number
}

export default function Home() {
    let [stats, setStats] = useState<Stats | undefined>(undefined);
    let [statsLoading, setStatsLoading] = useState<boolean>(true);

    useEffect(() => {
        fetch("https://localhost:3001/api/v1/stats")
            .then(response => {
                if (response.ok) {
                    return response.json();
                }
                throw response;
            })
            .then(data => setStats(data))
            .finally(() => setStatsLoading(false));
    }, []);

    return (
        <div className="grid items-_start gap-4 p-4 sm:p-6 md:gap-8">
            <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
                <CardStats
                    title="Stops" subtitle="Stop places accross datasets"
                    valueLoading={statsLoading}
                    value={stats?.num_stops?.toString()}
                    icon={<MapPin/>}/>
                <CardStats
                    title="Lines" subtitle="Lines accross datasets" value="4,202"
                    icon={<Waypoints/>}/>
                <CardStats
                    title="Trips" subtitle="Trips places accross datasets" value="34,502"
                    icon={<Navigation/>}/>
            </div>
            <div className="flex flex-col gap-4 md:gap-8">
                {STEPS.map((step) => (
                    <>
                        <h2 className="text-lg font-semibold leading-none tracking-tight flex items-center gap-3">
                            <LoadingSpinner size={18} /> {step.title}
                        </h2>
                        {step.cards.map((card) => (
                            <NavLink to={card.link} key={card.link}>
                                <Card className="flex flex-row items-center">
                                    <CardHeader className="bg-muted/50">
                                        <CardTitle>{card.title}</CardTitle>
                                        <CardDescription>{card.description}</CardDescription>
                                        <Button variant="link" className="gap-1">
                                            Learn more
                                            <ArrowUpRight className="w-4 h-4"/>
                                        </Button>
                                    </CardHeader>
                                </Card>
                            </NavLink>
                        ))}
                    </>
                ))}
            </div>
        </div>
    );
}