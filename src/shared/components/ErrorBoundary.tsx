import { Component, type ErrorInfo, type ReactNode } from 'react';

import { ErrorState } from '@shared/ui/ErrorState';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
}

/**
 * Catches the `Unexpected` category (ErrorHandling.md Section 2 "Unexpected" row) — anything
 * that reaches a component as a thrown JS error rather than a handled `AppError`. One instance
 * wraps each route-level page (CodingStandards.md Section 1).
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  override state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
  }

  override componentDidCatch(error: unknown, info: ErrorInfo): void {
    console.error('[ErrorBoundary]', error, info.componentStack);
  }

  private readonly handleReload = (): void => {
    this.setState({ hasError: false });
  };

  override render(): ReactNode {
    if (this.state.hasError) {
      return (
        <ErrorState description="Please try reloading this section." onRetry={this.handleReload} />
      );
    }
    return this.props.children;
  }
}
