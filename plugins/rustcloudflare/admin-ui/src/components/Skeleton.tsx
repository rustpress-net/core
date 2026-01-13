import { clsx } from 'clsx';

interface SkeletonProps {
  className?: string;
  variant?: 'text' | 'circular' | 'rectangular';
  width?: string | number;
  height?: string | number;
  count?: number;
}

export function Skeleton({
  className,
  variant = 'text',
  width,
  height,
  count = 1
}: SkeletonProps) {
  const baseClasses = 'animate-pulse bg-neutral-700/50';

  const variantClasses = {
    text: 'rounded',
    circular: 'rounded-full',
    rectangular: 'rounded-lg',
  };

  const style: React.CSSProperties = {
    width: width ? (typeof width === 'number' ? `${width}px` : width) : undefined,
    height: height ? (typeof height === 'number' ? `${height}px` : height) : undefined,
  };

  if (count > 1) {
    return (
      <div className="space-y-2">
        {Array.from({ length: count }).map((_, i) => (
          <div
            key={i}
            className={clsx(baseClasses, variantClasses[variant], className)}
            style={style}
          />
        ))}
      </div>
    );
  }

  return (
    <div
      className={clsx(baseClasses, variantClasses[variant], className)}
      style={style}
    />
  );
}

// Pre-built skeleton components for common patterns
export function StatsSkeleton() {
  return (
    <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
      <div className="flex items-center justify-between">
        <Skeleton variant="rectangular" width={48} height={48} />
        <Skeleton width={60} height={20} />
      </div>
      <div className="mt-4">
        <Skeleton width={100} height={32} className="mb-2" />
        <Skeleton width={80} height={16} />
      </div>
    </div>
  );
}

export function TableRowSkeleton({ columns = 4 }: { columns?: number }) {
  return (
    <tr>
      {Array.from({ length: columns }).map((_, i) => (
        <td key={i} className="px-6 py-4">
          <Skeleton width={i === 0 ? 150 : 80} height={20} />
        </td>
      ))}
    </tr>
  );
}

export function CardSkeleton() {
  return (
    <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
      <div className="flex items-center gap-3 mb-4">
        <Skeleton variant="rectangular" width={40} height={40} />
        <div className="flex-1">
          <Skeleton width={120} height={20} className="mb-2" />
          <Skeleton width={180} height={14} />
        </div>
      </div>
      <Skeleton count={3} height={16} className="mt-2" />
    </div>
  );
}

export function ChartSkeleton({ height = 300 }: { height?: number }) {
  return (
    <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
      <Skeleton width={180} height={24} className="mb-4" />
      <Skeleton variant="rectangular" height={height} className="w-full" />
    </div>
  );
}

export function ListSkeleton({ items = 5 }: { items?: number }) {
  return (
    <div className="space-y-3">
      {Array.from({ length: items }).map((_, i) => (
        <div key={i} className="flex items-center gap-3 p-3 bg-neutral-700/30 rounded-lg">
          <Skeleton variant="rectangular" width={40} height={40} />
          <div className="flex-1">
            <Skeleton width={150} height={18} className="mb-1" />
            <Skeleton width={100} height={14} />
          </div>
          <Skeleton width={60} height={24} />
        </div>
      ))}
    </div>
  );
}
