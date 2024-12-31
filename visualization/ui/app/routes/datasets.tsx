"use client"
import {Table, TableBody, TableCell, TableHead, TableHeader, TableRow} from "~/components/ui/table";
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "~/components/ui/card";
import {Badge} from "~/components/ui/badge";
import type {Route} from "../../.react-router/types/app/routes/+types/home";
import {useEffect, useState} from "react";
import {LoadingSpinner} from "~/components/ui/spinner";
import {Skeleton} from "~/components/ui/skeleton";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Datasets"},
    ];
}

interface Config {
    datasets: Dataset[],
    dataset_groups: any[]
}

interface Dataset {
    id: string;
    format: string;
    license: string;
    groups: string[];
    src: any;
}

interface DatasetGroup {
    id: string;
}

export default function DatasetsPage() {
    let [configLoaded, setConfigLoaded] = useState(false);
    let [datasets, setDatasets] = useState<Dataset[]>([]);
    let [datasetGroups, setDatasetGroups] = useState<DatasetGroup[]>([]);

    useEffect(() => {
        fetch("https://localhost:3001/api/v1/config")
            .then(response => {
                if (response.ok) {
                    return response.json();
                }

                throw response;
            })
            .then(data => {
                setDatasets(data.datasets);
                setDatasetGroups(data.dataset_groups);
            })
            .finally(() => setConfigLoaded(true));
    }, []);

    return (
        <>
            <div className="grid flex-1 items-_start gap-4 p-4 sm:p-6 md:gap-8">
                <Card>
                    <CardHeader>
                        <CardTitle>Imported Datasets</CardTitle>
                        <CardDescription>Inspect single datasets that are imported</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>
                                        ID
                                    </TableHead>
                                    <TableHead>
                                        Format
                                    </TableHead>
                                    <TableHead>
                                        License
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {configLoaded && datasets.map(dataset => (
                                    <TableRow onClick={() => {
                                        alert("on click")
                                    }}>
                                        <TableCell>
                                            {dataset.id}
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant="secondary">{dataset.format}</Badge>
                                        </TableCell>
                                        <TableCell>
                                            <Badge variant="outline">{dataset.license}</Badge>
                                        </TableCell>
                                    </TableRow>
                                ))}
                                {!configLoaded &&
                                    <>
                                        <TableRow>
                                            <TableCell colSpan={3} className="p-0">
                                                <Skeleton className="w-full h-9 rounded-none delay-0" />
                                            </TableCell>
                                        </TableRow>
                                        <TableRow>
                                            <TableCell colSpan={3} className="p-0">
                                                <Skeleton className="w-full h-9 rounded-none delay-200" />
                                            </TableCell>
                                        </TableRow>
                                        <TableRow>
                                            <TableCell colSpan={3} className="p-0">
                                                <Skeleton className="w-full h-9 rounded-none delay-400" />
                                            </TableCell>
                                        </TableRow>
                                    </>
                                }
                            </TableBody>
                        </Table>
                    </CardContent>
                    <CardFooter>
                        <div className="text-xs text-muted-foreground">
                            <b>{datasets.length} datasets</b> imported from config.yaml
                        </div>
                    </CardFooter>
                </Card>
                <Card>
                    <CardHeader>
                        <CardTitle>Dataset groups</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>Group ID</TableHead>
                                    <TableHead>Datasets</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {datasetGroups.map(group => (
                                    <TableRow>
                                        <TableCell>{group.id}</TableCell>
                                        <TableCell>
                                            <div className="flex flex-row gap-1">
                                            {datasets
                                                .filter(d => (d.groups.includes(group.id)))
                                                .map(d => (
                                                    <Badge variant="secondary">
                                                        {d.id}
                                                    </Badge>
                                                ))}
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </CardContent>
                    <CardFooter>
                        <div className="text-xs text-muted-foreground">
                            <b>{datasetGroups.length} dataset groups</b> imported from config.yaml
                        </div>
                    </CardFooter>
                </Card>
            </div>
        </>
    );
}
