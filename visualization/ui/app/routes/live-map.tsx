"use client"

import * as React from "react";
import {useEffect} from "react";

import {
    FullscreenControl,
    Map,
    NavigationControl,
    ScaleControl,
    useControl
} from "react-map-gl/dist/es5/exports-maplibre";
import {MapboxOverlay} from '@deck.gl/mapbox';
import {type DeckProps, type Position} from '@deck.gl/core';
import {IconLayer, ScatterplotLayer, TextLayer} from '@deck.gl/layers';
import 'maplibre-gl/dist/maplibre-gl.css';

import type {Route} from "../../.react-router/types/app/routes/+types/home";

export function meta({}: Route.MetaArgs) {
    return [
        {title: "Live Map"},
    ];
}

function DeckGLOverlay(props: DeckProps) {
    const overlay = useControl<MapboxOverlay>(() => new MapboxOverlay(props));
    overlay.setProps(props);
    return null;
}

type Vehicle = {
    name: string,
    position: Position,
    orientation: number,
};

const POSITION_UPDATE_INTERVAL = 250; // 4 times per second
const POSITION_TRANSITION = POSITION_UPDATE_INTERVAL * 3; // make transition slower to smooth out motions
const ROTATION_TRANSITION = POSITION_UPDATE_INTERVAL * 0.5;


export default function LiveMapPage() {
    const [data, setData] = React.useState<Vehicle[]>([]);

    useEffect(() => {
        const interval = setInterval(() => {
            setData(currentData => {
                let vehicles: Vehicle[] = [];
                for (let i = 0; i < (currentData.length || 10_000) + 10; i++) {
                    let newPosition: Position;
                    let orientation: number;
                    if (currentData.length > i) {
                        const oldPosition = currentData[i].position;
                        newPosition = [oldPosition[0] + 0.005 * (0.5 - Math.random()), oldPosition[1] + 0.005 * (0.5 - Math.random())];
                        // TODO: Fix rotation in extreme coordinates (near poles)
                        const orientationRad = Math.atan((newPosition[1] - oldPosition[1]) / (newPosition[0] - oldPosition[0]));
                        orientation = (orientationRad / (2 * Math.PI)) * 360 - 90;
                        if (oldPosition[0] - newPosition[0] > 0) {
                            orientation -= 180;
                        }
                    } else {
                        newPosition = [Math.random() * 180, Math.random() * 90];
                        orientation = Math.random() * 360;
                    }
                    vehicles.push({
                        name: "S1",
                        position: newPosition,
                        orientation
                    });
                }
                return vehicles;
            });
        }, POSITION_UPDATE_INTERVAL);

        return () => clearInterval(interval);
    }, []);

    const layers = [
        new ScatterplotLayer<Vehicle>({
            id: "vehicles",
            data,
            getPosition: (v: Vehicle) => v.position,

            stroked: true,
            getLineColor: [0, 0, 0, 200],
            getLineWidth: 3,
            lineWidthMaxPixels: 2,
            getRadius: 100,
            radiusMinPixels: 2,
            radiusMaxPixels: 12,
            getFillColor: v => [255, 100, 50, 255],

            transitions: {
                getPosition: POSITION_TRANSITION
            },
        }),
        new IconLayer<Vehicle>({
            id: "vehicles-direction-arrows",
            data,
            getPosition: (v: Vehicle) => v.position,
            getAngle: (v: Vehicle): number => v.orientation,

            getColor: (v: Vehicle) => [255, 100, 50, 255],
            getIcon: (_) => 'default',
            iconAtlas: "/assets/live-map/direction-icon-atlas.png",
            iconMapping: "/assets/live-map/direction-icon-mapping.json",

            sizeScale: 100,
            sizeMaxPixels: 128,
            sizeMinPixels: 5,
            billboard: false,

            transitions: {
                getPosition: POSITION_TRANSITION,
                getAngle: ROTATION_TRANSITION,
            },
        }),
        new TextLayer<Vehicle>({
            id: "vehicle-names",
            data,
            getPosition: (v: Vehicle) => v.position,

            getText: (v: Vehicle) => v.name,
            getAlignmentBaseline: 'center',
            getColor: [50, 20, 0, 255],
            getSize: 12,
            getTextAnchor: 'middle',
            billboard: false,

            transitions: {
                getPosition: POSITION_TRANSITION
            },
        })
        /*new ScatterplotLayer<Vehicle>({
            id: "vehicles-jumping",
            data,
            getPosition: (s: Vehicle): Position => ([s.lon, s.lat]),

            stroked: true,
            getLineColor: [0, 0, 0, 100],
            getLineWidth: 3,
            lineWidthMaxPixels: 3,
            getRadius: 14,
            radiusMinPixels: 2,
            radiusMaxPixels: 10,
        }),*/
    ];

    return (
        <Map
            initialViewState={{
                longitude: 0,
                latitude: 0,
                zoom: 1
            }}
            mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
            style={{width: "100%", height: "80vh"}}>

            <NavigationControl position="top-right"/>
            <FullscreenControl position="top-right"/>
            <ScaleControl/>

            <DeckGLOverlay
                layers={layers}
                controller/>

        </Map>
    );
}
