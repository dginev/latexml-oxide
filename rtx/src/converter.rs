use core::DigestionAPI;
use rtx_core::common::error::*;
use rtx_core::common::{Config, DataSize, OutputFormat};
use rtx_core::document::Document;
use rtx_core::list::List;
use rtx_core::state::State;
use rtx_core::token;
use rtx_core::{BoxOps, Core, Digested};

const CONVERTER_IDENTITY: &str = "rtx (v0.1)";

pub struct ConversionResponse {
  pub result: Option<String>,
  pub log: String,
  pub status: String,
  pub status_code: usize,
}
pub struct Runtime {
  pub status: String,
  pub status_code: usize,
}
pub struct Converter {
  runtime: Runtime,
  ready: bool,
  opts: Config,
  core: Core,
}

impl Converter {
  pub fn from_config(opts: Config) -> Converter {
    Converter {
      runtime: Runtime {
        status: String::new(),
        status_code: 3,
      },
      ready: false,
      opts,
      core: Core::default(),
    }
  }
  pub fn initialize_session(&mut self) -> Result<()> {
    // Prepare LaTeXML object
    self.core.initialize_state(vec![s!("TeX.pool")])?;
    self.ready = true;
    Ok(())
  }
  pub fn state_mut(&mut self) -> &mut State { self.core.state_mut() }
  pub fn bind_log(&mut self) {
    // TODO
  }
  pub fn flush_log(&mut self) -> String {
    // TODO
    s!("mock flush log")
  }

  pub fn convert(mut self, source: String) -> ConversionResponse {
    // 1 Prepare for conversion
    // 1.1 Initialize session if needed:
    if !self.ready {
      if let Err(e) = self.initialize_session() {
        // We can't initialize, return error:
        e.log_fatal();
      }
      if !self.ready {
        return ConversionResponse {
          result: None,
          log: self.flush_log(),
          status: s!("Initialization failed."),
          status_code: 3,
        };
      }
    }

    self.bind_log();
    // 1.2 Inform of identity, increase conversion counter
    if self.opts.verbosity >= 0 {
      info!("{}", CONVERTER_IDENTITY);
      // info!( "invoked as [$0 " . join(' ', @ARGV) . "]\n" if $$opts{verbosity} >= 1;
      // info!("processing started " . localtime() . "\n"; )
    }

    // 1.3 Prepare for What's IN:
    // - We use a new temporary variable to avoid confusion with daemon caching
    // - Math needs to magically trigger math mode if needed
    // - Fragments need to have a default pre- and postamble, if none provided
    let current_preamble = match self.opts.whatsin {
      DataSize::Math => Some(s!("literal:\\begin{{document}}\\ensuremathfollows")),
      DataSize::Fragment => match self.opts.preamble.clone() {
        Some(p) => Some(p.clone()),
        None => Some(s!("standard_preamble.tex")),
      },
      _ => None,
    };
    let current_postamble = match self.opts.whatsout {
      DataSize::Math => Some(s!("literal:\\ensuremathpreceeds\\end{{document}}")),
      DataSize::Fragment => match self.opts.postamble.clone() {
        Some(p) => Some(p),
        None => Some(s!("standard_postamble.tex")),
      },
      _ => None,
    };
    // TODO:
    // 1.3.3 Archives need to get unpacked in a sandbox (with sufficient bookkeeping)
    //   elsif ($$opts{whatsin} =~ /^archive/) {
    //     // Sandbox the input
    //     $$opts{archive_sourcedirectory} = $$opts{sourcedirectory};
    //     my $sandbox_directory = File::Temp->newdir(TMPDIR => 1);
    //     $$opts{sourcedirectory} = $sandbox_directory;
    //     // Extract the archive in the sandbox
    //     $source = unpack_source($source, $sandbox_directory);
    //     if (!defined $source) {    // Unpacking failed to find a source
    //       $$opts{sourcedirectory} = $$opts{archive_sourcedirectory};
    //       my $log = $self->flush_log;
    // return { result => undef, log => $log, status => "Fatal:IO:Archive Can't detect a
    // source TeX file!", status_code => 3 }; } // Destination magic: If we expect an archive
    // on output, we need to invent the appropriate destination ourselves when not given.
    // // Since the LaTeXML API never writes the final archive file to disk, we just use a pretend
    // sourcename.zip:     if (($$opts{whatsout} =~ /^archive/) && (!$$opts{destination})) {
    //       $$opts{placeholder_destination} = 1;
    //       $$opts{destination}             = pathname_name($source) . ".zip"; } }

    //   // 1.4 Prepare for What's OUT (if we need a sandbox)
    //   if ($$opts{whatsout} =~ /^archive/) {
    //     $$opts{archive_sitedirectory} = $$opts{sitedirectory};
    //     $$opts{archive_destination}   = $$opts{destination};
    // my $destination_name = $$opts{destination} ? pathname_name($$opts{destination}) :
    // 'document';     my $sandbox_directory = File::Temp->newdir(TMPDIR => 1);
    //     my $extension = $$opts{format};
    //     $extension =~ s/\d+$//;
    //     $extension =~ s/^epub|mobi$/xhtml/;
    //     my $sandbox_destination = "$destination_name.$extension";
    //     $$opts{sitedirectory} = $sandbox_directory;

    //     if ($$opts{format} eq 'epub') {
    //       $$opts{resource_directory} = File::Spec->catdir($sandbox_directory, 'OPS');
    // $$opts{destination} = pathname_concat(File::Spec->catdir($sandbox_directory, 'OPS'),
    // $sandbox_destination); }     else {
    //       $$opts{destination} = pathname_concat($sandbox_directory, $sandbox_destination); }
    //   }

    // 1.5 Prepare a daemon frame
    // self.core.pushDaemonFrame();
    //  withState(sub {
    //     my ($state) = @_;    // Sandbox state
    //     $state->pushDaemonFrame;
    //     $state->assignValue('_authlist', $$opts{authlist}, 'global');
    //     $state->assignValue('REMOTE_REQUEST', (!$$opts{local}), 'global');
    // });

    // 2 Beginning Core conversion - digest the source:
    // my ($digested, $dom, $serialized) = (undef, undef, undef);
    // Should be this, but is overridden by withState.
    // local $SIG{'ALRM'} = sub { LaTeXML::Common::Error::Fatal('conversion','timeout',
    // "Conversion timed out after " . $$opts{timeout} . " seconds!\n"); };
    // alarm($$opts{timeout});
    // my $mode = ($$opts{type} eq 'auto') ? 'TeX' : $$opts{type};
    let digest_result = self.core.digest(
      source,
      current_preamble,
      current_postamble,
      self.opts.mode.clone(),
      true,
    );
    let digested = match digest_result {
      Err(e) => {
        // TODO digestion failed, report
        e.log_fatal();
        Digested::List(Box::new(List::new(Vec::new())))
      },
      Ok(d) => d,
    };
    // 2.1 Now, convert to DOM and output, if desired.
    let dom_result: Result<Document>;
    let serialized = match self.opts.format {
      OutputFormat::TeX => token::untex(&digested, &self.core.state),
      OutputFormat::Box => {
        if self.opts.verbosity > 0 {
          digested.stringify()
        } else {
          digested.to_string()
        }
      },
      _ => {
        dom_result = self.core.convert_document(digested);
        match dom_result {
          Ok(dom) => dom.to_string(self.state_mut()),
          Err(e) => {
            error!(target: "document:convert", "{:?}", e);
            s!("Fatal: convert document failed")
          },
        }
      },
    };

    let status_code = self.core.state.status_code;
    self.runtime.status_code = status_code;
    // alarm(0)

    // 2.2 Bookkeeping in case fatal errors occurred
    // $$latexml{state}->noteStatus('fatal') if $latexml && $@;    // Fatal Error?
    // local $@ = 'Fatal:conversion:unknown TeX to XML conversion failed! (Unknown Reason)' if
    // ((!$convert_eval_return) && (!$@)); my $eval_report = $@;
    // $$runtime{status}      = $latexml->getStatusMessage;
    // $$runtime{status_code} = $latexml->getStatusCode;
    // $$runtime{status_data}->{$_} = $$latexml{state}->{status}->{$_} foreach (qw(warning error
    // fatal)); // End daemon run, by popping frame:
    // $latexml->withState(sub {
    //     my ($state) = @_;                                       // Remove current state frame
    // $$opts{searchpaths} = $state->lookupValue('SEARCHPATHS'); // save the searchpaths for
    // post-processing     $state->popDaemonFrame;
    //     $$state{status} = {};
    // });
    // if ($eval_report || ($$runtime{status_code} == 3)) {
    //   // Terminate immediately on Fatal errors
    //   $$runtime{status_code} = 3;
    //   log!($eval_report . "\n" if $eval_report);
    //   log!("\nConversion complete: " . $$runtime{status} . ".\n");
    //   log!("Status:conversion:" . ($$runtime{status_code} || '0') . "\n");
    //   // If we just processed an archive, clean up sandbox directory.
    //   if ($$opts{whatsin} =~ /^archive/) {
    //     rmtree($$opts{sourcedirectory});
    //     $$opts{sourcedirectory} = $$opts{archive_sourcedirectory}; }

    // Close and restore STDERR to original condition.
    // let log = self.flush_log();
    // $serialized = $dom->to_string if ($dom && (!defined $serialized));
    // $self->sanitize($log);

    // return { result => $serialized, log => $log, status => $$runtime{status}, status_code =>
    // $$runtime{status_code} }; } else {
    // Standard report, if we're not in a Fatal case
    // log!("\nConversion complete: " . $$runtime{status} . ".\n"; );

    // 2.3 Clean up and exit if we only wanted the serialization of the core conversion
    // if ($serialized) {
    //   // If serialized has been set, we are done with the job
    //   // If we just processed an archive, clean up sandbox directory.
    //   if ($$opts{whatsin} =~ /^archive/) {
    //     rmtree($$opts{sourcedirectory});
    //     $$opts{sourcedirectory} = $$opts{archive_sourcedirectory}; }
    //   my $log = $self->flush_log;
    // return { result => $serialized, log => $log, status => $$runtime{status}, status_code =>
    // $$runtime{status_code} }; }

    // 3 If desired, post-process
    // my $result = $dom;
    // if ($$opts{post} && $dom && $dom->documentElement) {
    //   my $post_eval_return = eval {
    //     local $SIG{'ALRM'} = sub { die "alarm\n" };
    //     alarm($$opts{timeout});
    //     $result = $self->convert_post($dom);
    //     alarm(0);
    //     1;
    //   };
    //   // 3.1 Bookkeeping if a post-processing Fatal error occurred
    //   //// $$latexml{state}->noteStatus('fatal') if $latexml && $@; // Fatal Error?
    //   local $@ = 'Fatal:conversion:unknown Post-processing failed! (Unknown Reason)'
    //     if ((!$post_eval_return) && (!$@));
    //   if ($@) {    //Fatal occured!
    //     $$runtime{status_code} = 3;
    //     $@ = 'Fatal:conversion:unknown '.$@ unless $@ =~ /^Fatal:/;
    //     error!($@);
    //     //Since this is postprocessing, we don't need to do anything
    //     //   just avoid crashing...
    //     $result = undef; } }

    // // 4 Clean-up: undo everything we sandboxed
    // if ($$opts{whatsin} =~ /^archive/) {
    //   rmtree($$opts{sourcedirectory});
    //   $$opts{sourcedirectory} = $$opts{archive_sourcedirectory}; }
    // if ($$opts{whatsout} =~ /^archive/) {
    //   rmtree($$opts{sitedirectory});
    //   $$opts{sitedirectory} = $$opts{archive_sitedirectory};
    //   $$opts{destination}   = $$opts{archive_destination};
    //   if (delete $$opts{placeholder_destination}) {
    //     delete $$opts{destination}; } }

    // // 5 Output
    // // 5.1 Serialize the XML/HTML result (or just return the Perl object, if requested)
    // undef $serialized;
    // if ((defined $result) && ref($result) && (ref($result) =~ /^(:?LaTe)?XML/)) {
    //   if (($$opts{format} =~ 'x(ht)?ml') || ($$opts{format} eq 'jats')) {
    //     $serialized = $result->to_string(1); }
    //   elsif ($$opts{format} =~ /^html/) {
    //     if (ref($result) =~ '^LaTeXML::(Post::)?Document$') {    // Special for documents
    //       $serialized = $result->getDocument->to_stringHTML; }
    //     else {                                                   // Regular for fragments
    //       do {
    //         local $XML::LibXML::setTagCompression = 1;
    //         $serialized = $result->to_string(1);
    //         } } }
    //   elsif ($$opts{format} eq 'dom') {
    //     $serialized = $result; } }
    // else { $serialized = $result; }                              // Compressed case

    // 5.2 Finalize logging and return a response containing the document result, log and status
    info!("status:conversion: {:?}", self.runtime.status_code);
    let log = self.flush_log();
    // self->sanitize($log) if ($$runtime{status_code} == 3);

    ConversionResponse {
      result: Some(serialized),
      log,
      status: self.runtime.status.clone(),
      status_code: self.runtime.status_code,
    }
  }

  pub fn prepare_session<'preplifetime>(
    &'preplifetime mut self,
    _opts: &'preplifetime Config,
  ) -> Result<()>
  {
    if !self.ready {
      self.initialize_session()?
    }
    Ok(())
  }
}
