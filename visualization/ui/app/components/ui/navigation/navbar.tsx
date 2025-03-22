import * as React from "react"

import {
    ArrowLeftRight,
    ChartScatter,
    CheckCircle,
    Database,
    Home,
    Import,
    Map,
    Route,
    ToggleRight,
    Waypoints
} from "lucide-react"
import {NavLink, useLocation} from "react-router";
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem
} from "~/components/ui/sidebar";
import {Progress} from "~/components/ui/progress";
import Drino from "~/components/ui/icon/drino";
import {GroupNode} from "@luma.gl/engine";


const NAV_BAR_GROUPS = [
    {
        items: [
            {
                title: "Home",
                icon: Home,
                link: "/",
            }
        ]
    },
    {
        groupTitle: "Setup",
        items: [
            {
                title: "Datasets",
                icon: Database,
                link: "/datasets",
            },
            {
                title: "Features",
                icon: ToggleRight,
                link: "/features",
            }
        ]
    },
    {
        groupTitle: "Preprocessing",
        items: [
            {
                title: "Imported Data",
                icon: Import,
                link: "/harvest-data",
            },
            {
                title: "Data validation",
                icon: CheckCircle,
                link: "/data-validation",
            },
            {
                title: "Clustering",
                icon: ChartScatter,
                link: "/clusters",
            },
            {
                title: "Transfer patterns",
                icon: Waypoints,
                link: "/transfer-patterns",
            }
        ]
    },
    {
        groupTitle: "Server",
        items: [
            {
                title: "Routing",
                icon: Route,
                link: "/routing",
            },
            {
                title: "Live Map",
                icon: Map,
                link: "/live-map",
            },
            {
                title: "APIs",
                icon: ArrowLeftRight,
                link: "/api-doc",
            }
        ]
    }
]

const NavBar = () => {
    const location = useLocation();

    return (
        <Sidebar collapsible="icon">
            <SidebarHeader>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton size="lg" asChild>
                            <NavLink to="/">
                                <div
                                    className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                                    <Drino className="size-4"/>
                                </div>
                                <div className="grid flex-1 text-left text-sm leading-tight">
                                    <span className="font-bold text-lg">Drino</span>
                                </div>
                            </NavLink>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarHeader>
            <SidebarContent>
                {NAV_BAR_GROUPS.map((group) => (
                    <SidebarGroup key={group.groupTitle}>
                        {group.groupTitle && <SidebarGroupLabel>{group.groupTitle}</SidebarGroupLabel>}
                        <SidebarGroupContent>
                            <SidebarMenu>
                                {group.items.map((item) => (
                                    <SidebarMenuItem key={item.link}>
                                        <SidebarMenuButton isActive={location?.pathname === item.link} asChild>
                                            <NavLink to={item.link}>
                                                <item.icon />
                                                {item.title}
                                            </NavLink>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                ))}
                            </SidebarMenu>
                        </SidebarGroupContent>
                    </SidebarGroup>
                ))}
            </SidebarContent>

            <SidebarFooter>
                <SidebarGroup>
                    <SidebarGroupLabel>Status: Preprocessing</SidebarGroupLabel>
                    <div className="m-2 flex flex-row items-baseline gap-4">
                        <span className="font-semibold">1/5</span>
                        <Progress value={22}/>
                    </div>
                </SidebarGroup>
            </SidebarFooter>
        </Sidebar>
        /*<aside className="fixed inset-y-0 left-0 z-10 hidden w-14 flex-col border-r bg-background sm:flex">
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
        </aside>*/
    )
}

NavBar.displayName = "NavBar"

export {NavBar}
