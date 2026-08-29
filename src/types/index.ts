export interface DeskEvent {
  id: string;
  title: string;
  description: string;
  event_time: string;
  completed: number;
  remind_at: string | null;
  remind_on_time: number;
  recurrence: string;
  recurrence_end: string | null;
  generated_next_id?: string | null;
  created_at: string;
  updated_at: string;
}

export type EventInput = Pick<
  DeskEvent,
  'title' | 'description' | 'event_time' | 'remind_at' | 'remind_on_time' | 'recurrence' | 'recurrence_end'
>;

export interface DataStatus {
  configured: boolean;
  dataDir: string | null;
  legacyDatabaseFound: boolean;
  error: string | null;
}

export interface ReminderRecord {
  id: number;
  event_id: string;
  fire_at: string;
  fired: number;
  type: string;
}

export type FilterTab = 'today' | 'upcoming' | 'completed' | 'all';

export interface ThemeSettings {
  windowOpacity: number;
  accentColor: string;
}

export interface AppSettings {
  defaultRemindOnTime: number;
}
