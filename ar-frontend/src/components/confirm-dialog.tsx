import {
  Alert,
  Button,
  Modal,
  ModalClose,
  ModalDialog,
  Stack,
  Typography,
} from "@mui/joy";
import { ErrorResponse } from "../network/fetch";

export function ConfirmDialog({
  isOpen,
  onClose,
  title,
  description,
  onCancelText,
  isActionPending,
  onSubmit,
  onSubmitText,
  isDanger,
  error,
}: {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  description?: string;
  onCancelText?: string;
  isActionPending: boolean;
  onSubmit: (() => Promise<void>) | (() => void);
  onSubmitText?: string;
  isDanger: boolean;
  error: ErrorResponse | null;
}) {
  console.log("hereeee???", error);
  return (
    <Modal open={isOpen} onClose={onClose}>
      <ModalDialog size="lg">
        <Typography level="h4">{title}</Typography>
        {error && (
          <Alert color="danger">
            <Typography color="danger">{error.message}</Typography>
          </Alert>
        )}
        <Typography>{description}</Typography>

        <Stack spacing={1} direction="row">
          <Button color="neutral" variant="outlined" onClick={onClose}>
            {onCancelText}
          </Button>
          <Button
            disabled={Boolean(isActionPending)}
            onClick={onSubmit}
            variant="solid"
            color={isDanger ? "danger" : "primary"}
          >
            {onSubmitText}
          </Button>
        </Stack>
        <ModalClose />
      </ModalDialog>
    </Modal>
  );
}
