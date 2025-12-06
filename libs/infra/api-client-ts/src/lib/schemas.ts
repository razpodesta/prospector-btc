import { z } from 'zod';

// =================================================================
// CONTRATOS DE DATOS (Mirror de domain-models-rs)
// =================================================================

export const WorkerHeartbeatSchema = z.object({
  worker_id: z.string().uuid(),
  hostname: z.string(),
  hashrate: z.number().int().nonnegative(),
  current_job_id: z.string().uuid().nullable().optional(),
  timestamp: z.string().datetime(), // Rust chrono serializa a ISO 8601
});

export const WorkOrderSchema = z.object({
  id: z.string().uuid(),
  strategy: z.union([
    z.literal('Random'),
    z.object({ Sequential: z.object({ start: z.string(), end: z.string() }) }),
    z.object({ Dictionary: z.object({ category: z.string() }) }),
  ]),
  target_duration_sec: z.number(),
});

// Tipos inferidos automáticamente
export type WorkerHeartbeat = z.infer<typeof WorkerHeartbeatSchema>;
export type WorkOrder = z.infer<typeof WorkOrderSchema>;
