import { taskDateKey } from "./dates";
import type { Task } from "./types";

export type TaskVisualState = {
  completed: boolean;
  overdue: boolean;
  dueToday: boolean;
  important: boolean;
  localPending: boolean;
};

export function classifyTaskVisualState(
  task: Pick<Task, "status" | "dueDateTime" | "timeZone" | "importance" | "dirty">,
  today = localTodayKey(),
): TaskVisualState {
  const completed = task.status === "completed";
  const dueDate = taskDateKey(task.dueDateTime, task.timeZone);

  return {
    completed,
    overdue: !completed && Boolean(dueDate) && dueDate < today,
    dueToday: !completed && Boolean(dueDate) && dueDate === today,
    important: task.importance === "high",
    localPending: Boolean(task.dirty),
  };
}

export function localTodayKey(now = new Date()) {
  const month = `${now.getMonth() + 1}`.padStart(2, "0");
  const day = `${now.getDate()}`.padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
}
