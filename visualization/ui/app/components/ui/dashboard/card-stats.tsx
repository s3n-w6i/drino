"use client"
import * as React from "react"

import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
} from "~/components/ui/card"
import {cn} from "~/lib/utils";
import {Skeleton} from "~/components/ui/skeleton";

export interface CardStatsProps extends React.HTMLAttributes<HTMLDivElement> {
    title: string,
    subtitle: string,
    value?: string | undefined,
    valueLoading?: boolean,
    icon: React.ReactElement
}

const CardStats = React.forwardRef<HTMLDivElement, CardStatsProps>(
    ({ className, title, subtitle, value, valueLoading, icon, ...props }, ref) => (
        <Card ref={ref}
              className={className}
              {...props}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{title}</CardTitle>
                {React.cloneElement(icon, { className: cn("h-5 w-5 text-muted-foreground", className) })}
            </CardHeader>
            <CardContent>
                {!valueLoading && <div className="text-2xl font-bold">{value}</div>}
                {valueLoading && <Skeleton className="max-w-full w-32 h-6 my-1" />}
                {subtitle && <p className="text-xs text-muted-foreground">{subtitle}</p>}
            </CardContent>
        </Card>
    )
)

CardStats.displayName = "CardStats";

export { CardStats }