import {NavBar} from "~/components/ui/navigation/navbar";
import ReactRouterBreadcrumbs from "~/components/ui/navigation/react-router-breadcrumbs";
import {Outlet} from "react-router";
import * as React from "react";

export default function SidebarLayout() {
    return (
        <div className="flex min-h-screen w-full flex-col">
            <NavBar/>
            <div className="flex h-screen flex-col sm:pt-4 sm:pl-14">
                <main className="flex-column flex-1 items-_start">
                    <div className="p-4 sm:p-6 md:gap-8">
                        <ReactRouterBreadcrumbs/>
                    </div>
                    <Outlet/>
                </main>
                <footer className="bg-background py-4 md:px-8 md:py-0">
                    <div className="container flex flex-col items-center justify-between gap-4 md:h-16 md:flex-row">
                        <p className="text-xs text-muted-foreground leading-loose">
                            drino v0.1
                        </p>
                    </div>
                </footer>
            </div>
        </div>
    )
}