"use client"
import * as React from "react"

import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
} from "~/components/ui/card"
import {cn} from "~/lib/utils";

export interface CardStatsProps extends React.HTMLAttributes<HTMLDivElement> {
    title: string,
    subtitle: string,
    value?: string,
    icon: React.ReactElement
}

const CardStats = React.forwardRef<HTMLDivElement, CardStatsProps>(
    ({ className, title, subtitle, value, icon, ...props }, ref) => (
        <Card ref={ref}
              className={className}
              {...props}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{title}</CardTitle>
                {React.cloneElement(icon, { className: cn("h-5 w-5 text-muted-foreground", className) })}
            </CardHeader>
            <CardContent>
                <div className="text-2xl font-bold">{value}</div>
                {subtitle && <p className="text-xs text-muted-foreground">{subtitle}</p>}
            </CardContent>
        </Card>
    )
)

CardStats.displayName = "CardStats";

export { CardStats }