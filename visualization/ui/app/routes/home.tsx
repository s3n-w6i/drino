import type {Route} from "./+types/home";
import {ArrowUpRight, CheckIcon, MapPin, Navigation, Waypoints, XIcon} from "lucide-react";
import {CardStats} from "~/components/ui/dashboard/card-stats";
import {NavLink} from "react-router";
import {Card, CardDescription, CardHeader, CardTitle} from "~/components/ui/card";
import {Button} from "~/components/ui/button";
import {useEffect, useState} from "react";
import {LoadingSpinner} from "~/components/ui/spinner";
import {Skeleton} from "~/components/ui/skeleton";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Home"},
    ];
}

interface Job {
    title: string;
    id: string;
    status: JobStatus;
    cards: JobCard[];
}

enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

interface JobCard {
    title: string;
    description: string;
    link: string;
}

const JOBS: Job[] = [
    {
        title: "Harvesting data",
        id: "HarvestData",
        status: JobStatus.Queued,
        cards: []
    },
    {
        title: "Importing data",
        id: "ImportData",
        status: JobStatus.Queued,
        cards: []
    },
    {
        title: "Validating data",
        id: "ValidateData",
        status: JobStatus.Queued,
        cards: []
    },
    {
        title: "Clustering",
        id: "PreprocessingClustering",
        status: JobStatus.Queued,
        cards: [
            {title: "Clustered stops", description: "Explore the calculated clustering", link: "map"}
        ]
    },
    {
        title: "Pre-Calculate connections within clusters",
        id: "PreprocessingLocalTransferPatterns",
        status: JobStatus.Queued,
        cards: []
    },
    {
        title: "Pre-Calculate connections (Long-distance)",
        id: "PreprocessingLongDistanceTransferPatterns",
        status: JobStatus.Queued,
        cards: []
    }
]

interface Stats {
    num_stops: number,
    num_trips: number
}

export default function Home() {
    let [jobs, setJobs] = useState<Job[]>(JOBS);

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

    useEffect(() => {
        const sse = new EventSource('https://localhost:3001/api/v1/status');

        sse.onerror = () => {
            console.error("sse error");
            setJobs([]);
            sse.close();
        };

        sse.onmessage = (msg) => {
            const data = JSON.parse(msg.data);

            for (const [jobId, newStatus] of Object.entries(data)) {
                const newJobs = jobs.map((j) => {
                    if (j.id === jobId) {
                        j.status = JobStatus[newStatus as keyof typeof JobStatus];
                    }
                    return j;
                });

                setJobs(newJobs);
            }
        }

        return () => {
            sse.close();
        };
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
                    title="Lines" subtitle="Lines accross datasets"
                    valueLoading={statsLoading}
                    value="42,4242"
                    icon={<Waypoints/>}/>
                <CardStats
                    title="Trips" subtitle="Trips places accross datasets"
                    valueLoading={statsLoading}
                    value={stats?.num_trips?.toString()}
                    icon={<Navigation/>}/>
            </div>
            <div className="flex flex-col gap-4 md:gap-8">
                {JOBS.map((job) => (
                    <div key={job.id}>
                        <h2 className="text-lg font-semibold leading-none tracking-tight flex items-center gap-3 h-5">
                            {job.status === JobStatus.Running && <LoadingSpinner size={20}/>}
                            {job.status === JobStatus.Queued && <Skeleton className="bg-gray-200 w-5 h-5 rounded-full" />}
                            {job.status === JobStatus.Succeeded && <div className="bg-green-600 w-5 h-5 rounded-full">
                                <CheckIcon className="w-3 h-full mx-auto text-green-100" />
                            </div>}
                            {job.status === JobStatus.Failed && <div className="bg-destructive w-5 h-5 rounded-full">
                                <XIcon className="w-3 h-full mx-auto text-red-100" />
                            </div>}
                            {job.title}
                        </h2>
                        {job.cards.map((card) => (
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
                    </div>
                ))}
            </div>
        </div>
    );
}