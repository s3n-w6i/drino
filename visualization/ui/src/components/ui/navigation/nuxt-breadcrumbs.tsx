"use client"

import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator
} from "@/components/ui/breadcrumb";
import Link from "next/link";
import { usePathname } from "next/navigation";

const PATH_TITLES = [
    { "dashboard": "Dashboard" },
    { "map": "Map", "datasets": "Datasets" }
]

export default function NuxtBreadcrumbs() {
    const path = usePathname();
    const pathPieces = path
        // trim leading and trailing slashes
        .replace(/^\/+/, '').replace(/\/+$/, '')
        .split("/");

    return (
        <Breadcrumb>
            <BreadcrumbList>
                {pathPieces.map((piece, index) => {
                    let path = "/" + pathPieces.slice(0, index + 1).join("/");
                    if (index < pathPieces.length - 1) {
                        return (
                            <>
                                <BreadcrumbItem key={path}>
                                    <BreadcrumbLink asChild>
                                        <Link href={path}>
                                            {PATH_TITLES[index][piece]}
                                        </Link>
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
