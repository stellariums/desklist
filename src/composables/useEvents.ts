import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import type { DeskEvent, FilterTab } from '../types';

async function getDb() {
  return await Database.load('sqlite:desklist.db');
}

export function useEvents() {
  const events = ref<DeskEvent[]>([]);
  const loading = ref(false);

  async function fetchEvents(filter: FilterTab = 'all') {
    loading.value = true;
    try {
      const db = await getDb();
      const now = new Date().toISOString();
      const todayStart = new Date();
      todayStart.setHours(0, 0, 0, 0);
      const todayEnd = new Date();
      todayEnd.setHours(23, 59, 59, 999);

      let query = '';
      let params: any[] = [];

      switch (filter) {
        case 'today':
          query = 'SELECT * FROM events WHERE event_time >= $1 AND event_time <= $2 AND completed = 0 ORDER BY event_time ASC';
          params = [todayStart.toISOString(), todayEnd.toISOString()];
          break;
        case 'upcoming':
          query = 'SELECT * FROM events WHERE event_time > $1 AND completed = 0 ORDER BY event_time ASC';
          params = [now];
          break;
        case 'completed':
          query = 'SELECT * FROM events WHERE completed = 1 ORDER BY updated_at DESC';
          break;
        case 'all':
        default:
          query = 'SELECT * FROM events ORDER BY event_time ASC';
          break;
      }

      events.value = await db.select<DeskEvent[]>(query, params);
    } finally {
      loading.value = false;
    }
  }

  async function createEvent(event: Omit<DeskEvent, 'id' | 'created_at' | 'updated_at'>) {
    const db = await getDb();
    const id = crypto.randomUUID();
    const now = new Date().toISOString();

    await db.execute(
      `INSERT INTO events (id, title, description, event_time, completed, remind_at, remind_on_time, recurrence, recurrence_end, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)`,
      [id, event.title, event.description || '', event.event_time, 0, event.remind_at, event.remind_on_time ?? 1, event.recurrence || 'none', event.recurrence_end, now, now]
    );

    await generateReminders(db, id, event.event_time, event.remind_at, event.remind_on_time ?? 1);
    return id;
  }

  async function updateEvent(id: string, event: Partial<DeskEvent>) {
    const db = await getDb();
    const now = new Date().toISOString();

    const fields: string[] = [];
    const values: any[] = [];
    let paramIndex = 1;

    if (event.title !== undefined) { fields.push(`title = $${paramIndex++}`); values.push(event.title); }
    if (event.description !== undefined) { fields.push(`description = $${paramIndex++}`); values.push(event.description); }
    if (event.event_time !== undefined) { fields.push(`event_time = $${paramIndex++}`); values.push(event.event_time); }
    if (event.remind_at !== undefined) { fields.push(`remind_at = $${paramIndex++}`); values.push(event.remind_at); }
    if (event.remind_on_time !== undefined) { fields.push(`remind_on_time = $${paramIndex++}`); values.push(event.remind_on_time); }
    if (event.recurrence !== undefined) { fields.push(`recurrence = $${paramIndex++}`); values.push(event.recurrence); }
    if (event.recurrence_end !== undefined) { fields.push(`recurrence_end = $${paramIndex++}`); values.push(event.recurrence_end); }

    fields.push(`updated_at = $${paramIndex++}`);
    values.push(now);
    values.push(id);

    await db.execute(`UPDATE events SET ${fields.join(', ')} WHERE id = $${paramIndex}`, values);

    if (event.event_time !== undefined || event.remind_at !== undefined || event.remind_on_time !== undefined) {
      await db.execute('DELETE FROM reminder_queue WHERE event_id = $1 AND fired = 0', [id]);
      const current = await db.select<DeskEvent[]>('SELECT * FROM events WHERE id = $1', [id]);
      if (current.length > 0) {
        const e = current[0];
        await generateReminders(db, id, e.event_time, e.remind_at, e.remind_on_time);
      }
    }
  }

  async function deleteEvent(id: string) {
    const db = await getDb();
    await db.execute('DELETE FROM reminder_queue WHERE event_id = $1', [id]);
    await db.execute('DELETE FROM events WHERE id = $1', [id]);
  }

  async function toggleComplete(id: string) {
    const db = await getDb();
    const now = new Date().toISOString();
    const rows = await db.select<DeskEvent[]>('SELECT * FROM events WHERE id = $1', [id]);
    if (rows.length === 0) return;

    const event = rows[0];
    const newCompleted = event.completed ? 0 : 1;
    await db.execute('UPDATE events SET completed = $1, updated_at = $2 WHERE id = $3', [newCompleted, now, id]);

    if (newCompleted === 1 && event.recurrence !== 'none') {
      const nextTime = calculateNextOccurrence(event.event_time, event.recurrence);
      if (nextTime && (!event.recurrence_end || nextTime <= event.recurrence_end)) {
        const newId = crypto.randomUUID();
        await db.execute(
          `INSERT INTO events (id, title, description, event_time, completed, remind_at, remind_on_time, recurrence, recurrence_end, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 0, $5, $6, $7, $8, $9, $10)`,
          [newId, event.title, event.description, nextTime, event.remind_at, event.remind_on_time, event.recurrence, event.recurrence_end, now, now]
        );
        await generateReminders(db, newId, nextTime, event.remind_at, event.remind_on_time);
      }
    }
  }

  async function generateReminders(db: any, eventId: string, eventTime: string, remindAt: string | null, remindOnTime: number) {
    if (remindOnTime) {
      await db.execute(
        'INSERT INTO reminder_queue (event_id, fire_at, fired, type) VALUES ($1, $2, 0, $3)',
        [eventId, eventTime, 'on_time']
      );
    }
    if (remindAt) {
      await db.execute(
        'INSERT INTO reminder_queue (event_id, fire_at, fired, type) VALUES ($1, $2, 0, $3)',
        [eventId, remindAt, 'advance']
      );
    }
  }

  function calculateNextOccurrence(eventTime: string, recurrence: string): string | null {
    const date = new Date(eventTime);
    switch (recurrence) {
      case 'daily': date.setDate(date.getDate() + 1); break;
      case 'weekly': date.setDate(date.getDate() + 7); break;
      case 'monthly': date.setMonth(date.getMonth() + 1); break;
      default: return null;
    }
    return date.toISOString();
  }

  return { events, loading, fetchEvents, createEvent, updateEvent, deleteEvent, toggleComplete };
}
