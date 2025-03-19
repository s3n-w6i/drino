import * as React from "react"

import {Tooltip, TooltipContent, TooltipTrigger} from "~/components/ui/tooltip";

import {Database, Home, Map, Route, Settings} from "lucide-react"
import Drino from "~/components/ui/icon/drino";
import {NavLink, useLocation} from "react-router";


const NAV_BAR_ITEMS = [
    {
        link: "/",
        icon: <Home/>,
        title: "Home"
    },
    {
        link: "/map",
        icon: <Map/>,
        title: "Map"
    },
    {
        link: "/routing",
        icon: <Route />,
        title: "Routing",
    },
    {
        link: "/datasets",
        icon: <Database/>,
        title: "Datasets"
    }
]

const NavBar = React.forwardRef<
    HTMLDivElement,
    React.HTMLAttributes<HTMLDivElement>
>(() => {
        const location = useLocation();

        return (
            <aside className="fixed inset-y-0 left-0 z-10 hidden w-14 flex-col border-r bg-background sm:flex">
                <nav className="flex flex-col items-center gap-4 px-2 py-4">
                    <NavLink
                        to="/"
                        className="group flex h-9 w-9 shrink-0 items-center justify-center gap-2 rounded-full bg-primary text-lg font-semibold text-primary-foreground md:h-8 md:w-8 md:text-base"
                    >
                        <Drino className="h-4 w-4 transition-all group-hover:scale-110"/>
                        <span className="sr-only">drino Dashboard</span>
                    </NavLink>
                    {NAV_BAR_ITEMS.map(({link, icon, title}) => (
                        <Tooltip key={link}>
                            <TooltipTrigger asChild>
                                <NavLink
                                    to={link}
                                    className={
                                        (link == location?.pathname) ? "flex h-9 w-9 items-center justify-center rounded-lg bg-accent text-accent-foreground transition-colors hover:text-foreground md:h-8 md:w-8"
                                            : "flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:text-foreground md:h-8 md:w-8"
                                    }>
                                    {React.cloneElement(icon, {className: "h-5 w-5"})}
                                    <span className="sr-only">{title}</span>
                                </NavLink>
                            </TooltipTrigger>
                            <TooltipContent side="right">{title}</TooltipContent>
                        </Tooltip>
                    ))}
                </nav>
                <nav className="mt-auto flex flex-col items-center gap-4 px-2 py-4">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <NavLink
                                to="settings"
                                className="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:text-foreground md:h-8 md:w-8"
                            >
                                <Settings className="h-5 w-5"/>
                                <span className="sr-only">Settings</span>
                            </NavLink>
                        </TooltipTrigger>
                        <TooltipContent side="right">Settings</TooltipContent>
                    </Tooltip>
                </nav>
            </aside>
        )
    }
)

NavBar.displayName = "NavBar"

export {NavBar}
