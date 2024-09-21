"use client"
import {Table, TableBody, TableCell, TableHead, TableHeader, TableRow} from "@/components/ui/table";
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "@/components/ui/card";
import {Badge} from "@/components/ui/badge";

export default function DatasetsPage() {
    return (
        <>
            <div className="grid flex-1 items-start gap-4 p-4 sm:p-6 md:gap-8">
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
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <TableRow onClick={() => { alert("clickidoo") }}>
                                    <TableCell>
                                        de:vvs:gtfs
                                    </TableCell>
                                    <TableCell>
                                        <Badge variant="secondary">GTFS</Badge>
                                    </TableCell>
                                </TableRow>
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
