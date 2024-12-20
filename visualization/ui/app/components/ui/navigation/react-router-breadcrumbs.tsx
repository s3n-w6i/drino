"use client"

import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator
} from "~/components/ui/breadcrumb";
import {NavLink, useLocation} from "react-router";

const PATH_TITLES: Record<string, string>[] = [
    { "map": "Map", "datasets": "Datasets" }
]

export default function ReactRouterBreadcrumbs() {
    const location = useLocation();
    const pathPieces = location.pathname
        // trim leading and trailing slashes
        .replace(/^\/+/, '').replace(/\/+$/, '')
        .split("/");

    return (
        <Breadcrumb>
            <BreadcrumbList>
                {pathPieces?.map((piece, index) => {
                    const path = "/" + pathPieces.slice(0, index + 1).join("/");
                    if (index < pathPieces.length - 1) {
                        return (
                            <>
                                <BreadcrumbItem key={path}>
                                    <BreadcrumbLink asChild>
                                        <NavLink to={path}>
                                            {PATH_TITLES[index][piece]}
                                        </NavLink>
                                    </BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator key={path + "/"}/>
                            </>
                        )
                    } else {
                        return (
                            <BreadcrumbItem key={path}>
                                <BreadcrumbPage>
                                    {PATH_TITLES[index][piece]}
                                </BreadcrumbPage>
                            </BreadcrumbItem>
                        )
                    }
                })}
            </BreadcrumbList>
        </Breadcrumb>
    );
}
