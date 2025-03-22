import {Popover, PopoverContent, PopoverTrigger} from "~/components/ui/popover";
import {Button} from "~/components/ui/button";
import {cn} from "~/lib/utils";
import {CalendarIcon} from "lucide-react";
import {format} from "date-fns/format";
import {Calendar} from "~/components/ui/calendar";
import React from "react";
import type {SelectSingleEventHandler} from "react-day-picker";

export interface DatePickerProps
    extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    date?: Date;
    onDateChange?: SelectSingleEventHandler;
    placeholder?: string;
}

const DatePicker = React.forwardRef<
    HTMLButtonElement,
    DatePickerProps
>((
    {className, date, onDateChange, placeholder = "Pick a date", ...props}, ref
) => {
    return (
        <Popover>
            <PopoverTrigger asChild>
                <Button
                    ref={ref}
                    variant="outline"
                    className={cn(
                        "w-[240px] justify-start text-left font-normal",
                        !date && "text-muted-foreground",
                        className
                    )}
                    {...props}
                >
                    <CalendarIcon/>
                    {date ? format(date, "PPP") : <span>{placeholder}</span>}
                </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto p-0" align="start">
                <Calendar
                    mode="single"
                    selected={date}
                    onSelect={onDateChange}
                    initialFocus
                />
            </PopoverContent>
        </Popover>
    )
})

DatePicker.displayName = "DatePicker";

interface DateTimePickerProps extends
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    DatePickerProps
{}

/*const DateTimePicker = React.forwardRef<
    HTMLButtonElement,
    DateTimePickerProps
>((
    { className, placeholder = "Pick a date & time", ...props}, ref
) => {
    return (

    )
})*/

export { DatePicker };
