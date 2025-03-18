import {type ClassValue, clsx} from "clsx"
import {twMerge} from "tailwind-merge"
import {toast} from "sonner";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

export function fetchData<T>(url: string): Promise<T | undefined> {
    return fetch(url)
        .catch((err: Error) => {
            console.log(err);
            toast.error("Error while sending request", { description: err.message });
        })
        .then(res => res?.json())
}