import { dev } from "$app/environment";
import toast from "svelte-french-toast";

function success(msg: string) {
    if (dev) console.log(msg);
    toast.success(msg, {
        iconTheme: {
            primary: '#0FB5E4',
            secondary: '#214485'
        }
    });
}

function error(msg: string) {
    if (dev) console.error(msg);
    toast.error(msg, {
        iconTheme: {
            primary: '#ff5454',
            secondary: '#214485'
        }
    });
}

export const notification = {
    success,
    error
}