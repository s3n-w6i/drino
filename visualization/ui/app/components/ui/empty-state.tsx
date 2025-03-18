import * as React from "react";

export interface EmptyStateProps extends React.HTMLAttributes<HTMLDivElement> {
    icon: React.ReactElement,
    title: string,
    description: string,
}

const EmptyState = React.forwardRef<
    HTMLDivElement,
    EmptyStateProps
>((
    { className, icon, title, description, ...props }
) => {
    return (
        <div className="flex flex-col items-center justify-center h-[50vh] gap-6" {...props}>
            <div className="flex items-center justify-center w-20 h-20 bg-gray-200 rounded-full dark:bg-gray-800">
                {icon}
            </div>
            <div className="space-y-2 text-center">
                <h2 className="text-2xl font-bold">{title}</h2>
                <p className="text-gray-500 dark:text-gray-400">
                    {description}
                </p>
            </div>
        </div>
    )
})

EmptyState.displayName = "EmptyState";

export { EmptyState };