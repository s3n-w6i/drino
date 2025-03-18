"use client"
import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle} from "~/components/ui/card";
import {EmptyState} from "~/components/ui/empty-state";
import type {Route} from "../../.react-router/types/app/routes/+types/home";
import {useState} from "react";
import {Ellipsis} from "lucide-react";
import {Input} from "~/components/ui/input";
import {Button} from "~/components/ui/button";
import {z} from "zod"
import {useForm} from "react-hook-form";
import {zodResolver} from "@hookform/resolvers/zod";
import {Form, FormControl, FormField, FormItem, FormLabel} from "~/components/ui/form";
import {Tabs, TabsList, TabsTrigger} from "~/components/ui/tabs";
import {LoadingSpinner} from "~/components/ui/spinner";
import {Skeleton} from "~/components/ui/skeleton";
import {toast} from "sonner";
import {fetchData} from "~/lib/utils";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Routing"},
    ];
}

type Query = {
    origin: string;
    destination: string;
    datetime: Date;
}

interface State {}

class EnterQueryState implements State {}
class LoadingState implements State {}
class ResultState implements State {}

export default function RoutingPage() {
    const formSchema = z.object({
        origin: z.string(),
        destination: z.string(),
        datetime: z.date()
    });

    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            origin: "",
            destination: "",
            datetime: new Date(),
        }
    });

    const onSubmit = async (query: z.infer<typeof formSchema>) => {
        setState(new LoadingState());

        const res = await fetchData("http://localhost:8080")

        if (res) {
            setState(new ResultState());
        } else {
            setState(new EnterQueryState());
        }
    }

    const [state, setState] = useState<State>(new EnterQueryState());

    return (
        <div className="flex items-start flex-row mx-4 sm:mx-6 gap-4 mb-6">
            <Card className="w-96 h-auto">
                <CardHeader>
                    <CardTitle>Routing Playground</CardTitle>
                    <CardDescription>Run queries on the transit network</CardDescription>
                </CardHeader>
                <CardContent>
                    <Form {...form}>
                        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                            <div className="space-y-2">
                                <FormField
                                    control={form.control}
                                    name="origin"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>From</FormLabel>
                                            <FormControl>
                                                <Input {...field} placeholder="Departure Station" />
                                            </FormControl>
                                        </FormItem>
                                    )}/>
                                <FormField
                                    control={form.control}
                                    name="destination"
                                    render={({ field }) => (
                                        <FormItem>
                                            <FormLabel>To</FormLabel>
                                            <FormControl>
                                                <Input {...field} placeholder="Destination Station" />
                                            </FormControl>
                                        </FormItem>
                                    )}/>
                            </div>
                            <FormField
                                control={form.control}
                                name="datetime"
                                render={({ field }) => (
                                    <FormItem className="grow">
                                        <FormLabel>Date & Time</FormLabel>
                                        <FormControl>
                                            <Input {...field} />
                                        </FormControl>
                                    </FormItem>
                                )}/>
                            <div className="flex w-full items-end justify-between space-x-2">
                                <Tabs defaultValue="depart-at">
                                    <TabsList>
                                        <TabsTrigger value="depart-at">Departure</TabsTrigger>
                                        <TabsTrigger value="arrive-at">Arrival</TabsTrigger>
                                    </TabsList>
                                </Tabs>
                                <Button type="submit" className="min-w-24">
                                    {!(state instanceof LoadingState) ?
                                        <>Search</> :
                                        <LoadingSpinner />
                                    }
                                </Button>
                            </div>
                        </form>
                    </Form>
                </CardContent>
            </Card>
            <div className="grow">
                {state instanceof EnterQueryState && (
                    <EmptyState
                        icon={<Ellipsis />}
                        title={"No query"}
                        description={"Enter a query to get started"} />
                )}
                {(state instanceof LoadingState || state instanceof ResultState) && (
                    <div className="flex flex-col gap-2 w-full max-w-screen-md mx-auto">
                        {(state instanceof LoadingState) && (
                            <>
                                <Skeleton className="h-24 w-full delay-0" />
                                <Skeleton className="h-24 w-full delay-100" />
                                <Skeleton className="h-24 w-full delay-200" />
                            </>
                        )}
                        {(state instanceof ResultState) && (
                            <Card className="w-full">
                                <CardHeader>
                                    <CardTitle>Hi</CardTitle>
                                </CardHeader>
                            </Card>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
