import { Bell, CalendarDays, Check, MoreHorizontal, Repeat, Star } from "lucide-react";
import clsx from "clsx";
import { formatTaskDate } from "../lib/dates";
import { useI18n } from "../lib/i18n";
import { classifyTaskVisualState } from "../lib/taskVisualState";
import type { Task } from "../lib/types";

type Props = {
  task: Task;
  completing: boolean;
  onComplete: (taskId: string) => void;
  onOpenDetails: (task: Task) => void;
};

export function TaskItem({ task, completing, onComplete, onOpenDetails }: Props) {
  const { language, t } = useI18n();
  const { completed, overdue, dueToday, important, localPending } =
    classifyTaskVisualState(task);

  return (
    <li
      className={clsx(
        "task-item",
        completing && "task-item--completing",
        overdue && "task-item--overdue",
        dueToday && "task-item--due-today",
        important && "task-item--important",
        localPending && "task-item--pending",
        completed && "task-item--completed",
      )}
    >
      <button
        className="task-checkbox"
        aria-label={completed ? t("task.completedAria", { title: task.title }) : t("task.complete", { title: task.title })}
        disabled={completing || completed}
        onClick={() => onComplete(task.id)}
      >
        {completed ? <Check size={13} /> : null}
      </button>
      <span className="task-main">
        <span className="task-title">{task.title}</span>
        {important || task.dueDateTime || task.isReminderOn || task.recurrence ? (
          <span className="task-meta">
            {important ? (
              <span className="task-meta--important" title={t("task.important")}>
                <Star size={12} fill="currentColor" aria-hidden="true" />
                {t("task.important")}
              </span>
            ) : null}
            {task.dueDateTime ? (
              <span
                className={clsx(
                  overdue && "task-meta--overdue",
                  dueToday && "task-meta--due-today",
                )}
                title={t("task.dueDate")}
              >
                <CalendarDays size={12} />
                {dueToday
                  ? t("task.today")
                  : formatTaskDate(task.dueDateTime, task.timeZone, language)}
              </span>
            ) : null}
            {task.isReminderOn ? (
              <span title={t("task.reminder")}>
                <Bell size={12} />
              </span>
            ) : null}
            {task.recurrence ? (
              <span title={t("task.repeats")}>
                <Repeat size={12} />
              </span>
            ) : null}
          </span>
        ) : null}
      </span>
      {completed || localPending ? (
        <span className="task-badges">
          {completed ? <span className="task-state">{t("task.done")}</span> : null}
          {localPending ? <span className="task-pending">{t("task.pending")}</span> : null}
        </span>
      ) : null}
      <button className="task-more" title={t("task.details")} aria-label={t("task.details")} onClick={() => onOpenDetails(task)}>
        <MoreHorizontal size={16} />
      </button>
    </li>
  );
}
