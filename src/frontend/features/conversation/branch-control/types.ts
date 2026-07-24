export interface GitBranchOption {
  readonly name: string;
  readonly targetOid: string;
}

export interface GitBranchControlProps {
  readonly activeBranch: string;
  readonly branches: readonly GitBranchOption[];
  readonly isBusy?: boolean;
  readonly onApprove: (approvalId: string) => void;
  readonly onCreate: (branchName: string) => void;
  readonly onDeny: (approvalId: string) => void;
  readonly onSwitch: (branchName: string) => void;
  readonly pendingApprovalId?: string | null;
}
