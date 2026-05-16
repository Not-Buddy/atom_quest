import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useAuth } from "@/contexts/AuthContext";
import { listUsers, createUser, updateUser, deleteUser } from "@/lib/api";
import type { User, UserRole } from "@/lib/types";
import { DashboardLayout } from "@/components/DashboardLayout";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Plus, Edit, Trash2, Loader2, AlertCircle, Users } from "lucide-react";

const ROLE_BADGE_VARIANT: Record<UserRole, "default" | "success" | "accent"> = {
  employee: "default",
  manager: "success",
  admin: "accent",
};

export default function AdminUsers() {
  const { token } = useAuth();
  const queryClient = useQueryClient();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<User | null>(null);
  const [error, setError] = useState("");

  const [fullName, setFullName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [role, setRole] = useState<UserRole>("employee");
  const [departmentId, setDepartmentId] = useState("");
  const [managerId, setManagerId] = useState("");

  const {
    data: users = [],
    isLoading,
    isError,
  } = useQuery({
    queryKey: ["admin-users"],
    queryFn: () => listUsers(token!),
    enabled: !!token,
  });

  const createMutation = useMutation({
    mutationFn: (payload: {
      email: string;
      full_name: string;
      password: string;
      department_id: number | null;
      role: UserRole;
      manager_id: number | null;
    }) => createUser(token!, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-users"] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const updateMutation = useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: {
        email?: string;
        full_name?: string;
        password?: string;
        department_id?: number | null;
        role?: UserRole;
        manager_id?: number | null;
      };
    }) => updateUser(token!, id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-users"] });
      closeDialog();
    },
    onError: (err: Error) => setError(err.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => deleteUser(token!, id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin-users"] });
      setDeleteConfirm(null);
    },
    onError: (err: Error) => setError(err.message),
  });

  const openAddDialog = () => {
    setEditingUser(null);
    setFullName("");
    setEmail("");
    setPassword("");
    setRole("employee");
    setDepartmentId("");
    setManagerId("");
    setError("");
    setDialogOpen(true);
  };

  const openEditDialog = (user: User) => {
    setEditingUser(user);
    setFullName(user.full_name);
    setEmail(user.email);
    setPassword("");
    setRole(user.role);
    setDepartmentId(user.department_id != null ? String(user.department_id) : "");
    setManagerId(user.manager_id != null ? String(user.manager_id) : "");
    setError("");
    setDialogOpen(true);
  };

  const closeDialog = () => {
    setDialogOpen(false);
    setEditingUser(null);
    setError("");
  };

  const handleSave = () => {
    setError("");
    if (!fullName.trim() || !email.trim()) {
      setError("Name and email are required");
      return;
    }
    if (!editingUser && !password) {
      setError("Password is required for new users");
      return;
    }

    const deptId = departmentId.trim() ? Number(departmentId) : null;
    const mgrId = managerId.trim() ? Number(managerId) : null;

    if (editingUser) {
      const payload: Record<string, unknown> = {
        full_name: fullName.trim(),
        email: email.trim(),
        role,
        department_id: deptId,
        manager_id: mgrId,
      };
      if (password) payload.password = password;
      updateMutation.mutate({ id: editingUser.id, payload });
    } else {
      createMutation.mutate({
        email: email.trim(),
        full_name: fullName.trim(),
        password,
        role,
        department_id: deptId,
        manager_id: mgrId,
      });
    }
  };

  const isSaving = createMutation.isPending || updateMutation.isPending;

  return (
    <DashboardLayout>
      <div className="max-w-6xl mx-auto space-y-6">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div>
            <h1 className="text-2xl font-bold text-slate-100">Users</h1>
            <p className="text-sm text-slate-400 mt-1">Manage user accounts and roles</p>
          </div>
          <Button
            onClick={openAddDialog}
            className="bg-indigo-600 hover:bg-indigo-500 text-white"
          >
            <Plus className="mr-2 h-4 w-4" />
            Add User
          </Button>
        </div>

        {isError && (
          <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>Failed to load users</AlertDescription>
          </Alert>
        )}

        {isLoading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-indigo-500" />
          </div>
        ) : users.length === 0 ? (
          <Card className="bg-slate-900/80 backdrop-blur-sm border border-slate-800">
            <CardContent className="py-8 text-center">
              <Users className="h-8 w-8 text-slate-600 mx-auto mb-3" />
              <p className="text-slate-400">No users found</p>
            </CardContent>
          </Card>
        ) : (
          <div className="rounded-xl border border-slate-800 overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow className="border-slate-800 hover:bg-transparent">
                  <TableHead className="text-slate-400">Name</TableHead>
                  <TableHead className="text-slate-400">Email</TableHead>
                  <TableHead className="text-slate-400">Role</TableHead>
                  <TableHead className="text-slate-400">Department ID</TableHead>
                  <TableHead className="text-slate-400">Manager ID</TableHead>
                  <TableHead className="text-slate-400 text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((u) => (
                  <TableRow key={u.id} className="border-slate-800">
                    <TableCell className="font-medium text-slate-200">{u.full_name}</TableCell>
                    <TableCell className="text-slate-400">{u.email}</TableCell>
                    <TableCell>
                      <Badge variant={ROLE_BADGE_VARIANT[u.role]} className="capitalize text-xs">
                        {u.role}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-slate-400">{u.department_id ?? "—"}</TableCell>
                    <TableCell className="text-slate-400">{u.manager_id ?? "—"}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-slate-400 hover:text-indigo-400"
                          onClick={() => openEditDialog(u)}
                        >
                          <Edit className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-slate-400 hover:text-red-400"
                          onClick={() => setDeleteConfirm(u)}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogContent className="bg-slate-900 border border-slate-800 text-slate-100 max-w-md">
            <DialogHeader>
              <DialogTitle>{editingUser ? "Edit User" : "Add User"}</DialogTitle>
              <DialogDescription className="text-slate-400">
                {editingUser ? "Update user details." : "Create a new user account."}
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-3">
              {error && (
                <Alert variant="destructive" className="bg-red-950/50 border-red-800 text-red-400">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              <div className="space-y-1.5">
                <Label className="text-slate-300">Full Name</Label>
                <Input
                  value={fullName}
                  onChange={(e) => setFullName(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>
              <div className="space-y-1.5">
                <Label className="text-slate-300">Email</Label>
                <Input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>
              <div className="space-y-1.5">
                <Label className="text-slate-300">
                  Password {editingUser && "(leave blank to keep current)"}
                </Label>
                <Input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label className="text-slate-300">Role</Label>
                  <Select value={role} onValueChange={(v) => setRole(v as UserRole)}>
                    <SelectTrigger className="bg-slate-800/50 border-slate-700 text-slate-100">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent className="bg-slate-900 border-slate-700 text-slate-100">
                      <SelectItem value="employee">Employee</SelectItem>
                      <SelectItem value="manager">Manager</SelectItem>
                      <SelectItem value="admin">Admin</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1.5">
                  <Label className="text-slate-300">Department ID</Label>
                  <Input
                    type="number"
                    value={departmentId}
                    onChange={(e) => setDepartmentId(e.target.value)}
                    className="bg-slate-800/50 border-slate-700 text-slate-100"
                  />
                </div>
              </div>
              <div className="space-y-1.5">
                <Label className="text-slate-300">Manager ID</Label>
                <Input
                  type="number"
                  value={managerId}
                  onChange={(e) => setManagerId(e.target.value)}
                  className="bg-slate-800/50 border-slate-700 text-slate-100"
                />
              </div>
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                onClick={closeDialog}
                className="border-slate-700 text-slate-300"
              >
                Cancel
              </Button>
              <Button
                onClick={handleSave}
                disabled={isSaving}
                className="bg-indigo-600 hover:bg-indigo-500 text-white"
              >
                {isSaving ? <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" /> : null}
                {editingUser ? "Update" : "Create"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <AlertDialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
          <AlertDialogContent className="bg-slate-900 border border-slate-800 text-slate-100">
            <AlertDialogHeader>
              <AlertDialogTitle>Delete User</AlertDialogTitle>
              <AlertDialogDescription className="text-slate-400">
                Are you sure you want to delete {deleteConfirm?.full_name}? This action cannot be
                undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel className="border-slate-700 text-slate-300 bg-transparent">
                Cancel
              </AlertDialogCancel>
              <AlertDialogAction
                className="bg-red-600 hover:bg-red-500"
                onClick={() => deleteConfirm && deleteMutation.mutate(deleteConfirm.id)}
              >
                {deleteMutation.isPending ? (
                  <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" />
                ) : null}
                Delete
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    </DashboardLayout>
  );
}
