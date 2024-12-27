"use client"
import {Table, TableBody, TableCell, TableHead, TableHeader, TableRow} from "~/components/ui/table";
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "~/components/ui/card";
import {Badge} from "~/components/ui/badge";
import type {Route} from "../../.react-router/types/app/routes/+types/home";
import {useEffect, useState} from "react";

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

export default function DatasetsPage() {
    let [configLoaded, setConfigLoaded] = useState(false);
    let [datasets, setDatasets] = useState<Dataset[]>([]);

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
                                {datasets.map(dataset => (
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
                            </TableBody>
                        </Table>
                    </CardContent>
                    <CardFooter>
                        <div className="text-xs text-muted-foreground">
                            <b>1 dataset</b> imported from config.yaml
                        </div>
                    </CardFooter>
                </Card>
            </div>
        </>
    );
}
