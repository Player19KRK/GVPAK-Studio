const managed=new Set();
export function markManagedTask(task){if(task?.pipeline)managed.add(task.id);}
export const isManagedTask=id=>managed.has(id);
