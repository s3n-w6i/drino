"use client"
import {Table, TableBody, TableCell, TableHead, TableHeader, TableRow} from "@/components/ui/table";
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "@/components/ui/card";

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
                                        Head 1
                                    </TableHead>
                                    <TableHead>
                                        Head 2
                                    </TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <TableRow onClick={() => { alert("clickidoo") }}>
                                    <TableCell>
                                        Value
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </CardContent>
                    <CardFooter>
                        <div className="text-xs text-muted-foreground">
                            Hi
                        </div>
                    </CardFooter>
                </Card>
            </div>
        </>
    );
}
