import { invoke } from './tauri'
import { listen, type UnlistenFn } from './tauri'
import type { BatchOperation, CreateOperationInput, ValidateResult } from '@/types/operation'

export interface OperationProgressEvent {
  operationId: string
  repoId: string
  status: string
  current: number
  total: number
  error: string | null
}

export function createOperation(input: CreateOperationInput): Promise<BatchOperation> {
  return invoke('create_operation', { input })
}

export function runOperation(id: string): Promise<void> {
  return invoke('run_operation', { id })
}

export function abortOperation(id: string): Promise<void> {
  return invoke('abort_operation', { id })
}

export function listOperations(): Promise<BatchOperation[]> {
  return invoke('list_operations')
}

export function getOperation(id: string): Promise<BatchOperation> {
  return invoke('get_operation', { id })
}

export function validateOperation(
  packageName: string,
  targetVersion: string,
  repoIds: string[],
): Promise<ValidateResult[]> {
  return invoke('validate_operation', { packageName, targetVersion, repoIds })
}

export function rollbackOperation(id: string): Promise<void> {
  return invoke('rollback_operation', { id })
}

export function onOperationProgress(
  callback: (event: OperationProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<OperationProgressEvent>('operation-progress', (event) => {
    callback(event.payload)
  })
}
