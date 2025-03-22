"use client"
import {Card, CardContent, CardDescription, CardHeader, CardTitle} from "~/components/ui/card";
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
import {fetchData} from "~/lib/utils";
import {TabsContent} from "@radix-ui/react-tabs";
import {Checkbox} from "~/components/ui/checkbox";
import {DatePicker} from "~/components/ui/date-picker";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Routing"},
    ];
}


enum QueryType {
    EarliestArrival = "earliest-arrival",
    LatestDeparture = "latest-departure",
    Range = "range"
}

const EarliestArrivalQuerySchema = z.object({
    origin: z.string(),
    destination: z.string(),
    datetime: z.coerce.date(), // Ensures proper date parsing
});

const RangeQuerySchema = z.object({
    start: z.number(),
    target: z.number(),
    earliest_departure: z.coerce.date(), // Ensures proper date parsing
    range: z.number(),
});

const AnyQuerySchema = z.union([RangeQuerySchema, EarliestArrivalQuerySchema]);


type Result = {
    journeys: Journey[];
}

type Journey = {
    legs: Leg[];
}

interface Leg {
    leg: LegType;
}

enum LegType {
    RIDE = "ride",
    TRANSFER = "transfer",
}

interface State {
}

class EnterQueryState implements State {
}

class LoadingState implements State {
}

class ResultState implements State {
    result: Result;

    constructor(result: Result) {
        this.result = result;
    }
}

export default function RoutingPage() {

    const form = useForm<z.infer<typeof AnyQuerySchema>>({
        resolver: zodResolver(AnyQuerySchema),
        defaultValues: {
            origin: "",
            destination: "",
            datetime: new Date(),
        }
    });

    const onSubmit = async (query: z.infer<typeof AnyQuerySchema>) => {
        setState(new LoadingState());

        const res = await fetchData<Result>("http://localhost:8080/api/v1/routing?" + new URLSearchParams({
            start: query.origin.toString(),
            target_type: "all",
            earliest_departure: new Date().getUTCSeconds().toString(),
            range: String(70_000),
        }));

        if (res) {
            console.log(res);
            setState(new ResultState(res));
        } else {
            setState(new EnterQueryState());
        }
    }

    const [state, setState] = useState<State>(new EnterQueryState());
    const [queryType, setQueryType] = useState<QueryType>(QueryType.EarliestArrival);

    return (
        <div className="flex items-start flex-row mx-4 sm:mx-6 gap-4 mb-6">
            <Card className="w-96 h-auto">
                <CardHeader>
                    <CardTitle>Routing Playground</CardTitle>
                    <CardDescription>Run queries on the transit network</CardDescription>
                </CardHeader>
                <CardContent>
                    <Form {...form}>
                        <form
                            onSubmit={form.handleSubmit(onSubmit)}
                            className="space-y-4">
                            <Tabs
                                className="space-y-4"
                                defaultValue={QueryType.EarliestArrival}
                                onValueChange={(value) => setQueryType(value as QueryType)}>
                                <TabsList>
                                    <TabsTrigger value={QueryType.EarliestArrival}>Earliest Arrival</TabsTrigger>
                                    <TabsTrigger value={QueryType.LatestDeparture}>Latest Departure</TabsTrigger>
                                    <TabsTrigger value={QueryType.Range}>Range</TabsTrigger>
                                </TabsList>

                                <TabsContent value={QueryType.EarliestArrival} className="space-y-4">
                                    <div className="space-y-2">
                                        <FormField
                                            control={form.control}
                                            name="origin"
                                            render={({field}) => (
                                                <FormItem>
                                                    <FormLabel>From</FormLabel>
                                                    <FormControl>
                                                        <Input {...field} placeholder="Departure Station"/>
                                                    </FormControl>
                                                </FormItem>
                                            )}/>
                                        <div className="flex flex-row gap-2 items-center">
                                            <FormField
                                                control={form.control}
                                                name="destination"
                                                render={({field}) => (
                                                    <FormItem>
                                                        <FormLabel>To</FormLabel>
                                                        <FormControl>
                                                            <Input {...field} placeholder="Destination Station"/>
                                                        </FormControl>
                                                    </FormItem>
                                                )}/>
                                            <Checkbox id="all"/>
                                            <label
                                                htmlFor="all"
                                                className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                                            >
                                                All
                                            </label>
                                        </div>
                                    </div>
                                    <FormField
                                        control={form.control}
                                        name="datetime"
                                        render={({field}) => (
                                            <FormItem className="grow">
                                                <FormLabel>Departure</FormLabel>
                                                <FormControl>
                                                    <DatePicker className="w-full" />
                                                </FormControl>
                                            </FormItem>
                                        )}/>
                                </TabsContent>

                            </Tabs>

                            <Button type="submit" className="w-full">
                                {!(state instanceof LoadingState) ?
                                    <>Search</> :
                                    <LoadingSpinner/>
                                }
                            </Button>
                        </form>
                    </Form>
                </CardContent>
            </Card>
            <div className="grow">
                {state instanceof EnterQueryState && (
                    <EmptyState
                        icon={<Ellipsis/>}
                        title={"No query"}
                        description={"Enter a query to get started"}/>
                )}
                {(state instanceof LoadingState || state instanceof ResultState) && (
                    <div className="flex flex-col gap-2 w-full max-w-screen-md mx-auto">
                        {(state instanceof LoadingState) && (
                            <>
                                <Skeleton className="h-24 w-full delay-0"/>
                                <Skeleton className="h-24 w-full delay-100"/>
                                <Skeleton className="h-24 w-full delay-200"/>
                            </>
                        )}
                        {(state instanceof ResultState) && (
                            (state as ResultState).result.journeys.map((journey) => (
                                <Card key={journey} className="w-full">
                                    <CardHeader>
                                        <CardTitle>
                                            {(journey.legs).map((leg) => {
                                                switch (leg.leg) {
                                                    case LegType.RIDE:
                                                        return leg.start;
                                                    case LegType.TRANSFER:
                                                        return "Transfer";
                                                }
                                            })}
                                        </CardTitle>
                                    </CardHeader>
                                </Card>
                            ))
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
