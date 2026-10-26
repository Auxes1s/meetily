import React from 'react';
import { CheckCircle2, Loader2, XCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import type { PermissionRowProps } from '@/types/onboarding';

export function PermissionRow({ icon, title, description, status, isPending = false, onAction }: PermissionRowProps) {
  const isAuthorized = status === 'authorized';
  const isDenied = status === 'denied';
  const isChecking = isPending;

  const getButtonText = () => {
    if (isChecking) return 'Checking...';
    if (isDenied) return 'Open Settings';
    return 'Enable';
  };

  return (
    <div
      className={cn(
        'flex flex-wrap items-center justify-between gap-3 rounded-2xl border px-4 py-5 sm:px-6',
        'transition-all duration-200',
        isAuthorized ? 'border-primary/20 bg-primary/5' : isDenied ? 'border-red-300 bg-red-50' : 'paper-surface border-border'
      )}
    >
      {/* Left side: Icon + Info */}
      <div className="flex items-center gap-3 flex-1 min-w-0">
        {/* Icon */}
        <div
          className={cn(
            'flex size-10 items-center justify-center rounded-full flex-shrink-0',
            isAuthorized ? 'bg-primary/10' : isDenied ? 'bg-red-100' : 'bg-muted'
          )}
        >
          <div className={cn(isAuthorized ? 'text-primary' : isDenied ? 'text-red-500' : 'text-muted-foreground')}>{icon}</div>
        </div>

        {/* Title + Description */}
        <div className="min-w-0 flex-1">
          <div className="font-medium text-foreground">{title}</div>
          <div className="text-sm text-muted-foreground">
            {isAuthorized ? (
              <span className="text-primary flex items-center gap-1">
                <CheckCircle2 className="w-3.5 h-3.5" />
                Access Granted
              </span>
            ) : isDenied ? (
              <span className="text-red-500 flex items-center gap-1">
                <XCircle className="w-3.5 h-3.5" />
                Access Denied - Please grant in System Settings
              </span>
            ) : (
              <span>{description}</span>
            )}
          </div>
        </div>
      </div>

      {/* Right side: Action button or checkmark */}
      <div className="flex items-center gap-2 flex-shrink-0 ml-3">
        {!isAuthorized && (
          <Button
            variant={isDenied ? "destructive" : "outline"}
            size="sm"
            onClick={onAction}
            disabled={isChecking}
            className="min-w-[100px]"
          >
            {isChecking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {getButtonText()}
          </Button>
        )}
        {isAuthorized && (
          <div className="flex size-8 items-center justify-center rounded-full bg-primary/10">
            <CheckCircle2 className="w-4 h-4 text-primary" />
          </div>
        )}
      </div>
    </div>
  );
}
