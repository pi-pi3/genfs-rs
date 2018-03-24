//! genfs
//!
//! A set of traits for implementing generic unix-style filesystems withour
//! relying on an architecture or specific operating system.
//!
//! Function definitions are mostly copied from the Rust standard library, with
//! some minor changes. This crate doesn't depend on the standard library or the
//! `alloc` crate.
//!
//! Documentation is mostly copied from the Rust standard library.

#![no_std]
#![deny(missing_docs)]

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
///
/// [`Seek`]: trait.Seek.html
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Set the offset to the provided number of bytes.
    Start(u64),

    /// Set the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error
    /// to seek before byte 0.
    End(i64),

    /// Set the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error
    /// to seek before byte 0.
    Current(i64),
}

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file.
///
/// Generally speaking, when using `OpenOptions`, you'll first call [`new`],
/// then chain calls to methods to set each option, then call [`open`],
/// passing the path of the file you're trying to open. This will give you a
/// `Result` with a [`File`] inside that you can further
/// operate on.
///
/// [`new`]: struct.OpenOptions.html#method.new
/// [`open`]: struct.OpenOptions.html#method.open
/// [`File`]: trait.File.html
///
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct OpenOptions<Permissions> {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    mode: Permissions,
    flags: u32,
}

impl<Permissions: Default> OpenOptions<Permissions> {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    pub fn new() -> Self {
        OpenOptions::default()
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// For most filesystems, the operating system guarantees that all writes
    /// are atomic: no writes get mangled because another process writes at
    /// the same time.
    ///
    /// One maybe obvious note when using append-mode: make sure that all data
    /// that belongs together is written to the file in one operation. This
    /// can be done by concatenating strings before passing them to [`write()`],
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling [`flush()`] when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at
    /// the end of the file. So, before writing, save the current position
    /// (using [`seek`]`(`[`SeekFrom`]`::`[`Current`]`(0))`, and restore it
    /// before the next read.
    ///
    /// ## Note
    ///
    /// This function doesn't create the file if it doesn't exist. Use the
    /// [`create`] method to do so.
    ///
    /// [`write()`]: trait.File.html#method.write
    /// [`flush()`]: trait.File.html#method.flush
    /// [`seek`]: trait.File.html#method.seek
    /// [`SeekFrom`]: enum.SeekFrom.html
    /// [`Current`]: enum.SeekFrom.html#variant.Current
    /// [`create`]: trait.Fs.html#method.create
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option for creating a new file.
    ///
    /// This option indicates whether a new file will be created if the file
    /// does not yet already exist.
    ///
    /// In order for the file to be created, [`write`] or [`append`] access must
    /// be used.
    ///
    /// [`write`]: #method.write
    /// [`append`]: #method.append
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to always create a new file.
    ///
    /// This option indicates whether a new file will be created.
    /// No file is allowed to exist at the target location, also no (dangling)
    /// symlink.
    ///
    /// This option is useful because it is atomic. Otherwise between checking
    /// whether a file exists and creating a new one, the file may have been
    /// created by another process (a TOCTOU race condition / attack).
    ///
    /// If `.create_new(true)` is set, [`.create()`] and [`.truncate()`] are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// [`.create()`]: #method.create
    /// [`.truncate()`]: #method.truncate
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Sets the mode bits that a new file will be created with.
    pub fn mode(&mut self, mode: Permissions) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Pass custom flags to the `flags` argument of `open`.
    pub fn custom_flags(&mut self, flags: u32) -> &mut Self {
        self.flags = flags;
        self
    }
}

/// A builder used to create directories in various manners.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct DirOptions<Permissions> {
    recursive: bool,
    mode: Permissions,
    flags: u32,
}

impl<Permissions: Default> DirOptions<Permissions> {
    /// Creates a new set of options with default mode/security settings for all
    /// platforms and also non-recursive.
    pub fn new() -> Self {
        DirOptions::default()
    }

    /// Indicates that directories should be created recursively, creating all
    /// parent directories. Parents that do not exist are created with the same
    /// security and permissions settings.
    ///
    /// This option defaults to `false`.
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Sets the mode to create new directories with.
    pub fn mode(&mut self, mode: Permissions) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Pass custom flags to the `flags` argument of `open`.
    pub fn custom_flags(&mut self, flags: u32) -> &mut Self {
        self.flags = flags;
        self
    }
}

/// Filesystem manipulation operations.
///
/// This trait contains basic methods to manipulate the contents of the local
/// filesystem. All methods in this module represent cross-platform filesystem
/// operations.
pub trait Fs {
    /// The borrowed path slice that represents a relative or absolute path on
    /// the filesystem.
    type Path: ?Sized;
    /// The owned path that represents a relative or absolute path on
    /// the filesystem.
    type PathOwned;
    /// The type that represents a file on the filesystem.
    type File: File<Error = Self::Error>;
    /// The type that represents a directory on the filesystem.
    type Dir: Dir<Self::DirEntry, Self::Error>;
    /// The type that represents an entry in a directory on the filesystem.
    type DirEntry: DirEntry<
        Path = Self::Path,
        Metadata = Self::Metadata,
        Error = Self::Error,
    >;
    /// The type that represents the metadata on the filesystem.
    type Metadata;
    /// The type that represents the permissions of a reader/writer on the
    /// filesystem.
    type Permissions;
    /// The type that represents the set of all errors that can occur during
    /// reading or writing.
    type Error;

    /// Opens a file at `path` with the options specified by `options`.
    ///
    /// # Errors
    ///
    /// This function will return an error under a number of different
    /// circumstances.
    fn open(
        &self,
        path: &Self::Path,
        options: &OpenOptions<Self::Permissions>,
    ) -> Result<Self::File, Self::Error>;

    /// Removes a file from the filesystem.
    ///
    /// Note that there is no
    /// guarantee that the file is immediately deleted (e.g. depending on
    /// platform, other open file descriptors may prevent immediate removal).
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * `path` points to a directory.
    /// * The user lacks permissions to remove the file.
    fn remove_file(&mut self, path: &Self::Path) -> Result<(), Self::Error>;

    /// Given a path, query the file system to get information about a file,
    /// directory, etc.
    ///
    /// This function will traverse symbolic links to query information about
    /// the destination file.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The user lacks permissions to perform `metadata` call on `path`.
    /// * `path` does not exist.
    fn metadata(
        &self,
        path: &Self::Path,
    ) -> Result<Self::Metadata, Self::Error>;

    /// Query the metadata about a file without following symlinks.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The user lacks permissions to perform `metadata` call on `path`.
    /// * `path` does not exist.
    fn symlink_metadata(
        &self,
        path: &Self::Path,
    ) -> Result<Self::Metadata, Self::Error>;

    /// Rename a file or directory to a new name, replacing the original file if
    /// `to` already exists.
    ///
    /// This will not work if the new name is on a different mount point.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * `from` does not exist.
    /// * The user lacks permissions to view contents.
    /// * `from` and `to` are on separate filesystems.
    fn rename(
        &mut self,
        from: &Self::Path,
        to: &Self::Path,
    ) -> Result<(), Self::Error>;

    /// Copies the contents of one file to another. This function will also
    /// copy the permission bits of the original file to the destination file.
    ///
    /// This function will **overwrite** the contents of `to`.
    ///
    /// Note that if `from` and `to` both point to the same file, then the file
    /// will likely get truncated by this operation.
    ///
    /// On success, the total number of bytes copied is returned and it is
    /// equal to the length of the `to` file as reported by `metadata`.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The `from` path is not a file.
    /// * The `from` file does not exist.
    /// * The current process does not have the permission rights to access
    ///   `from` or write `to`.
    fn copy(
        &mut self,
        from: &Self::Path,
        to: &Self::Path,
    ) -> Result<u64, Self::Error>;

    /// Creates a new hard link on the filesystem.
    ///
    /// The `dst` path will be a link pointing to the `src` path. Note that
    /// systems often require these two paths to both be located on the
    /// same filesystem.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The `src` path is not a file or doesn't exist.
    fn hard_link(
        &mut self,
        src: &Self::Path,
        dst: &Self::Path,
    ) -> Result<(), Self::Error>;

    /// Creates a new symbolic link on the filesystem.
    ///
    /// The `dst` path will be a symbolic link pointing to the `src` path.
    fn symlink(
        &mut self,
        src: &Self::Path,
        dst: &Self::Path,
    ) -> Result<(), Self::Error>;

    /// Reads a symbolic link, returning the file that the link points to.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * `path` is not a symbolic link.
    /// * `path` does not exist.
    fn read_link(
        &self,
        path: &Self::Path,
    ) -> Result<Self::PathOwned, Self::Error>;

    /// Returns the canonical form of a path with all intermediate components
    /// normalized and symbolic links resolved.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * `path` does not exist.
    /// * A component in path is not a directory.
    fn canonicalize(
        &self,
        path: &Self::Path,
    ) -> Result<Self::PathOwned, Self::Error>;

    /// Creates a new, empty directory at the provided path with the specified
    /// options.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * User lacks permissions to create directory at `path`.
    /// * `path` already exists, unless the `recursive` options was set.
    fn create_dir(
        &mut self,
        path: &Self::Path,
        options: &DirOptions<Self::Permissions>,
    ) -> Result<(), Self::Error>;

    /// Removes an existing, empty directory.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The user lacks permissions to remove the directory at the provided
    /// `path`. * The directory isn't empty.
    fn remove_dir(&mut self, path: &Self::Path) -> Result<(), Self::Error>;

    /// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    ///
    /// This function does **not** follow symbolic links and it will simply
    /// remove the symbolic link itself.
    ///
    /// # Errors
    ///
    /// See `Fs::remove_file` and `Fs::remove_dir`.
    fn remove_dir_all(&mut self, path: &Self::Path) -> Result<(), Self::Error>;

    /// Returns an iterator over the entries within a directory.
    ///
    /// The iterator will yield instances of `Result``<`[`DirEntry`]`>`.
    /// New errors may be encountered after an iterator is initially
    /// constructed.
    ///
    /// [`DirEntry`]: trait.DirEntry.html
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * The provided `path` doesn't exist.
    /// * The process lacks permissions to view the contents.
    /// * The `path` points at a non-directory file.
    fn read_dir(&self, path: &Self::Path) -> Result<Self::Dir, Self::Error>;

    /// Changes the permissions found on a file or a directory.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is
    /// not limited to just these cases:
    ///
    /// * `path` does not exist.
    /// * The user lacks the permission to change attributes of the file.
    fn set_permissions(
        &mut self,
        path: &Self::Path,
        perm: Self::Permissions,
    ) -> Result<(), Self::Error>;
}

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with.
///
/// Files should be automatically closed when they go out of scope.
pub trait File {
    /// The type that represents the set of all errors that can occur during
    /// reading or writing.
    type Error;

    /// Pull some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// This function does not provide any guarantees about whether it blocks
    /// waiting for data, but if an object needs to block for a read but cannot
    /// it will typically signal this via an `Err` return value.
    ///
    /// If the return value of this method is `Ok(n)`, then it must be
    /// guaranteed that `0 <= n <= buf.len()`. A nonzero `n` value indicates
    /// that the buffer `buf` has been filled in with `n` bytes of data from
    /// this source. If `n` is `0`, then it can indicate one of two
    /// scenarios:
    ///
    /// 1. This reader has reached its "end of file" and will likely no longer
    ///    be able to produce bytes. Note that this does not mean that the
    ///    reader will *always* no longer be able to produce bytes.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// No guarantees are provided about the contents of `buf` when this
    /// function is called, implementations cannot rely on any property of the
    /// contents of `buf` being true. It is recommended that implementations
    /// only write data to `buf` instead of reading its contents.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O or other error, an error
    /// variant will be returned.
    fn read(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    /// Write a buffer into this object, returning how many bytes were written.
    ///
    /// This function will attempt to write the entire contents of `buf`, but
    /// the entire write may not succeed, or the write may also generate an
    /// error. A call to `write` represents *at most one* attempt to write to
    /// any wrapped object.
    ///
    /// Calls to `write` are not guaranteed to block waiting for data to be
    /// written, and a write which would otherwise block can be indicated
    /// through an `Err` variant.
    ///
    /// If the return value is `Ok(n)` then it must be guaranteed that
    /// `0 <= n <= buf.len()`. A return value of `0` typically means that the
    /// underlying object is no longer able to accept bytes and will likely not
    /// be able to in the future as well, or that the buffer provided is empty.
    ///
    /// # Errors
    ///
    /// Each call to `write` may generate an I/O error indicating that the
    /// operation could not be completed.
    ///
    /// It is **not** considered an error if the entire buffer could not be
    /// written to this writer.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// # Errors
    ///
    /// It is considered an error if not all bytes could be written due to
    /// I/O errors or EOF being reached.
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Seek to an offset, in bytes, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but implementation
    /// defined.
    ///
    /// If the seek operation completed successfully,
    /// this method returns the new position from the start of the stream.
    /// That position can be used later with [`SeekFrom::Start`].
    ///
    /// # Errors
    ///
    /// Seeking to a negative offset is considered an error.
    ///
    /// [`SeekFrom::Start`]: enum.SeekFrom.html#variant.Start
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error>;
}

/// Iterator over the entries in a directory.
///
/// This iterator is returned from the [`read_dir`] function of this module and
/// will yield instances of `Result<`[`DirEntry`]`>`. Through a
/// [`DirEntry`] information like the entry's path and possibly other metadata
/// can be learned.
///
/// # Errors
///
/// This `Result` will be an `Err` if there's some sort of intermittent
/// IO error during iteration.
///
/// [`read_dir`]: trait.Fs.html#method.read_dir
/// [`DirEntry`]: trait.DirEntry.html
pub trait Dir<T: DirEntry, E>: Iterator<Item = Result<T, E>> {}

/// Entries returned by the [`Dir`] iterator.
///
/// [`Dir`]: struct.Dir.html
///
/// An instance of `DirEntry` represents an entry inside of a directory on the
/// filesystem. Each entry can be inspected via methods to learn about the full
/// path or possibly other metadata through per-platform extension traits.
pub trait DirEntry {
    /// The borrowed path slice that represents a relative or absolute path on
    /// the filesystem.
    type Path: ?Sized;
    /// The owned path that represents a relative or absolute path on
    /// the filesystem.
    type PathOwned;
    /// The type that represents a files metadata on the filesystem.
    type Metadata;
    /// The type that represents the union of all possible filetypes.
    type FileType;
    /// The type that represents the set of all errors that can occur during
    /// reading or writing.
    type Error;

    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir`
    /// with the filename of this entry.
    fn path(&self) -> Self::PathOwned;

    /// Return the metadata for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    fn metadata(&self) -> Result<Self::Metadata, Self::Error>;

    /// Return the file type for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    fn file_type(&self) -> Result<Self::FileType, Self::Error>;

    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    fn file_name(&self) -> &Self::Path;
}
