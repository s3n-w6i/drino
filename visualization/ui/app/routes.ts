import {index, layout, prefix, route, type RouteConfig} from "@react-router/dev/routes";

export default [
    layout("layouts/sidebar.tsx", [
        index("routes/home.tsx"),
        route("map", "routes/map.tsx"),
        route("datasets", "routes/datasets.tsx"),
        route("live-map", "routes/live-map.tsx"),
        route("routing", "routes/routing.tsx"),
    ]),
] satisfies RouteConfig;