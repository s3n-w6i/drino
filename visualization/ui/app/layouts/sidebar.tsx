import {NavBar} from "~/components/ui/navigation/navbar";
import ReactRouterBreadcrumbs from "~/components/ui/navigation/react-router-breadcrumbs";
import {Outlet} from "react-router";
import * as React from "react";
import {SidebarInset, SidebarProvider, SidebarTrigger} from "~/components/ui/sidebar";
import {Separator} from "~/components/ui/separator";

// noinspection JSUnusedGlobalSymbols
export default function SidebarLayout() {
    const [sidebarOpen, setSidebarOpen] = React.useState(true);

    return (
        <SidebarProvider
            open={sidebarOpen}
            onOpenChange={(value) => setSidebarOpen(value)}
            defaultOpen={true}>
            <NavBar/>
            <SidebarInset className="flex-col">
                <header className="p-4 sm:p-5 flex flex-row items-center gap-2">
                    <SidebarTrigger/>
                    <Separator orientation="vertical" className="mr-2 h-4"/>
                    <ReactRouterBreadcrumbs/>
                </header>
                <Separator />
                <div className="grow flex flex-col sm:pt-2 bg-muted/80">
                    <Outlet/>
                </div>
                <Separator />
                <footer className="bg-background py-4 md:px-8 md:py-0">
                    <div className="container flex flex-col items-center justify-between gap-4 md:h-16 md:flex-row">
                        <p className="text-xs text-muted-foreground leading-loose">
                            drino v0.1
                        </p>
                    </div>
                </footer>
            </SidebarInset>
        </SidebarProvider>
    )
}