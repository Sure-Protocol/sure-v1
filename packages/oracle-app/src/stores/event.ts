import { writable } from 'svelte/store';

export type EventStatus = 'success' | 'error' | 'info';

export type Event = {
	name: string;
	message?: string;
	status: EventStatus;
	tx?: string;
};
export const newEvent = writable<Event>({ name: '', message: '', status: 'info', tx: '' }, () => {
	console.log('subscribe');
});
