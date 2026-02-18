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
  created_at: string;
  updated_at: string;
}

export interface ReminderRecord {
  id: number;
  event_id: string;
  fire_at: string;
  fired: number;
  type: string;
}

export type FilterTab = 'today' | 'upcoming' | 'completed' | 'all';
