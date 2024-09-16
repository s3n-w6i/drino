import {CardStats} from "@/components/ui/dashboard/card-stats";

import { MapPin, Waypoints, Navigation } from "lucide-react";

export default function HomePage() {
    return (
        <div className="grid items-start gap-4 p-4 sm:p-6 md:gap-8">
            <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4">
                <CardStats
                    title="Stops" subtitle="Stop places accross datasets" value="10,002"
                    icon={<MapPin />} />
                <CardStats
                    title="Routes" subtitle="Routes accross datasets" value="4,202"
                    icon={<Waypoints />} />
                <CardStats
                    title="Trips" subtitle="Trips places accross datasets" value="34,502"
                    icon={<Navigation />} />
            </div>
        </div>
    );
}
