// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreTypes.h"
#include "UObject/Interface.h"
#include "CurveIdentifier.h"
#include "Misc/FrameRate.h"
#include "IAnimationDataController.generated.h"

class UAssetUserData;

class UAnimDataModel;

namespace UE {
namespace Anim {
	class FOpenBracketAction;
	class FCloseBracketAction;
}}

/**
 * The Controller is the sole authority to perform changes on the Animation Data Model. Any mutation to the model made will
 * cause a subsequent notify (EAnimDataModelNotifyType) to be broadcasted from the Model's ModifiedEvent. Alongside of it is a 
 * payload containing information relevant to the mutation. These notifies should be relied upon to update any dependent views 
 * or generated (derived) data.
 */
UINTERFACE(BlueprintType, meta=(CannotImplementInterfaceInBlueprint))
class ENGINE_API UAnimationDataController : public UInterface
{
	GENERATED_BODY()
};

class ENGINE_API IAnimationDataController
{
public:
	GENERATED_BODY()

#if WITH_EDITOR
	/** RAII helper to define a scoped-based bracket, opens and closes a controller bracket automatically */
    struct FScopedBracket
	{
		FScopedBracket(IAnimationDataController* InController, const FText& InDescription)
            : Controller(*InController)
		{
			Controller.OpenBracket(InDescription);
		}

		FScopedBracket(IAnimationDataController& InController, const FText& InDescription)
            : Controller(InController)
		{
			Controller.OpenBracket(InDescription);
		}
		
		FScopedBracket(TScriptInterface<IAnimationDataController>& InController, const FText& InDescription)
            : Controller(*InController)
		{
			Controller.OpenBracket(InDescription);
		}
		
		~FScopedBracket()
		{
			Controller.CloseBracket();
		}
	private:
		IAnimationDataController& Controller;
	};
#endif // WITH_EDITOR

	/**
	* Sets the AnimDataModel instance this controller is supposed to be targeting
	*
	* @param	InModel		UAnimDataModel instance to target
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void SetModel(UAnimDataModel* InModel) = 0;

	/**
	* @return		The AnimDataModel instance this controller is currently targeting
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual UAnimDataModel* GetModel() = 0;

	/**
	* @return		The AnimDataModel instance this controller is currently targeting
	*/
	virtual const UAnimDataModel* const GetModel() const = 0;

	/**
	* Opens an interaction bracket, used for combining a set of controller actions. Broadcasts a EAnimDataModelNotifyType::BracketOpened notify,
	* this can be used by any Views or dependendent systems to halt any unnecessary or invalid operations until the (last) bracket is closed.
	*
	* @param	InTitle				Description of the bracket, e.g. "Generating Curve Data"
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void OpenBracket(const FText& InTitle, bool bShouldTransact = true) = 0;

	/**
	* Closes a previously opened interaction bracket, used for combining a set of controller actions. Broadcasts a EAnimDataModelNotifyType::BracketClosed notify.
	*
	* @param	InTitle				Description of the bracket, e.g. "Generating Curve Data"
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void CloseBracket(bool bShouldTransact = true) = 0;

	/**
	* Sets the total play-able length in seconds. Broadcasts a EAnimDataModelNotifyType::SequenceLengthChanged notify if successful.
	* The number of frames and keys for the provided length is recalculated according to the current value of UAnimDataModel::FrameRate.
	*
	* @param	Length				New play-able length value, has to be positive and non-zero
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void SetPlayLength(float Length, bool bShouldTransact = true) = 0;

	/*** Sets the total play-able length in seconds. Broadcasts a EAnimDataModelNotifyType::SequenceLengthChanged notify if successful.
	* T0 and T1 are expected to represent the window of time that was either added or removed. E.g. for insertion T0 indicates the time
	* at which additional time starts and T1 were it ends. For removal T0 indicates the time at which time should be started to remove, and T1 indicates the end. Giving a total of T1 - T0 added or removed length.
	* The number of frames and keys for the provided length is recalculated according to the current value of UAnimDataModel::FrameRate.
	* @param	Length				Total new play-able length value, has to be positive and non-zero
	* @param	T0					Point between 0 and Length at which the change in time starts
	* @param	T1					Point between 0 and Length at which the change in time ends
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void ResizePlayLength(float NewLength, float T0, float T1, bool bShouldTransact = true) = 0;

	/**
	* Sets the total play-able length in seconds and resizes curves. Broadcasts EAnimDataModelNotifyType::SequenceLengthChanged
	* and EAnimDataModelNotifyType::CurveChanged notifies if successful.
	* T0 and T1 are expected to represent the window of time that was either added or removed. E.g. for insertion T0 indicates the time
	* at which additional time starts and T1 were it ends. For removal T0 indicates the time at which time should be started to remove, and T1 indicates the end. Giving a total of T1 - T0 added or removed length.
	* The number of frames and keys for the provided length is recalculated according to the current value of UAnimDataModel::FrameRate.
	*
	* @param	Length				Total new play-able length value, has to be positive and non-zero
	* @param	T0					Point between 0 and Length at which the change in time starts
	* @param	T1					Point between 0 and Length at which the change in time ends
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void Resize(float Length, float T0, float T1, bool bShouldTransact = true) = 0;

	/**
	* Sets the frame rate according to which the bone animation is expected to be sampled. Broadcasts a EAnimDataModelNotifyType::FrameRateChanged notify if successful.
	* The number of frames and keys for the provided frame rate is recalculated according to the current value of UAnimDataModel::PlayLength.
	*
	* @param	FrameRate			The new sampling frame rate, has to be positive and non-zero
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void SetFrameRate(FFrameRate FrameRate, bool bShouldTransact = true) = 0;

	/**
	* Adds a new bone animation track for the provided name. Broadcasts a EAnimDataModelNotifyType::TrackAdded notify if successful.
	*
	* @param	BoneName			Bone name for which a track should be added
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	The index at which the bone track was added, INDEX_NONE if adding it failed
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual int32 AddBoneTrack(FName BoneName, bool bShouldTransact = true) = 0;

	/**
	* Inserts a new bone animation track for the provided name, at the provided index. Broadcasts a EAnimDataModelNotifyType::TrackAdded notify if successful.
	* The bone name is verified with the AnimModel's outer target USkeleton to ensure the bone exists.
	*
	* @param	BoneName			Bone name for which a track should be inserted
	* @param	DesiredIndex		Index at which the track should be inserted
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	The index at which the bone track was inserted, INDEX_NONE if the insertion failed
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual int32 InsertBoneTrack(FName BoneName, int32 DesiredIndex, bool bShouldTransact = true) = 0;

	/**
	* Removes an existing bone animation track with the provided name. Broadcasts a EAnimDataModelNotifyType::TrackRemoved notify if successful.
	*
	* @param	BoneName			Bone name of the track which should be removed
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the removal was succesful
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual bool RemoveBoneTrack(FName BoneName, bool bShouldTransact = true) = 0;

	/**
	* Removes all existing Bone Animation tracks. Broadcasts a EAnimDataModelNotifyType::TrackRemoved for each removed track, wrapped within BracketOpened/BracketClosed notifies.
	*
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual void RemoveAllBoneTracks(bool bShouldTransact = true) = 0;

	/**
	* Removes an existing bone animation track with the provided name. Broadcasts a EAnimDataModelNotifyType::TrackChanged notify if successful.
	* The provided number of keys provided is expected to match for each component, and be non-zero.
	*
	* @param	BoneName			Bone name of the track for which the keys should be set
	* @param	PositionalKeys		Array of keys for the translation component
	* @param	RotationalKeys		Array of keys for the rotation component
	* @param	ScalingKeys			Array of keys for the scale component
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the keys were succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = AnimationData)
	virtual bool SetBoneTrackKeys(FName BoneName, const TArray<FVector>& PositionalKeys, const TArray<FQuat>& RotationalKeys, const TArray<FVector>& ScalingKeys, bool bShouldTransact = true) = 0;
	
	/**
	* Adds a new curve with the provided information. Broadcasts a EAnimDataModelNotifyType::CurveAdded notify if successful.
	*
	* @param	CurveId				Identifier for the to-be-added curve
	* @param	CurveFlags			Flags to be set for the curve
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve was succesfully added
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool AddCurve(const FAnimationCurveIdentifier& CurveId, int32 CurveFlags = 0x00000004, bool bShouldTransact = true) = 0;

	/**
	* Duplicated the curve with the identifier. Broadcasts a EAnimDataModelNotifyType::CurveAdded notify if successful.
	*
	* @param	CopyCurveId			Identifier for the to-be-duplicated curve
	* @param	NewCurveId			Identifier for the to-be-added curve
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve was succesfully duplicated
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool DuplicateCurve(const FAnimationCurveIdentifier& CopyCurveId, const FAnimationCurveIdentifier& NewCurveId, bool bShouldTransact = true) = 0;
	

	/**
	* Remove the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveRemoved notify if successful.
	*
	* @param	CopyCurveId			Identifier for the to-be-removed curve
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve was succesfully removed
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool RemoveCurve(const FAnimationCurveIdentifier& CurveId, bool bShouldTransact = true) = 0;

	/**
	* Removes all the curves of the provided type. Broadcasts a EAnimDataModelNotifyType::CurveRemoved for each removed curve, wrapped within BracketOpened/BracketClosed notifies.
	*
	* @param	SupportedCurveType	Type for which all curves are to be removed
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual void RemoveAllCurvesOfType(ERawCurveTrackTypes SupportedCurveType, bool bShouldTransact = true) = 0;

	/**
	* Set an individual flag for the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveFlagsChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the curve for which the flag state is to be set
	* @param	Flag				Flag for which the state is supposed to be set
	* @param	bState				State of the flag to be, true=set/false=not set
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the flag state was succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetCurveFlag(const FAnimationCurveIdentifier& CurveId, EAnimAssetCurveFlags Flag, bool bState = true, bool bShouldTransact = true) = 0;

	/**
	* Replace the flags for the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveFlagsChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the curve for which the flag state is to be set
	* @param	Flags				Flag mask with which the existings flags are to be replaced
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the flag mask was succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetCurveFlags(const FAnimationCurveIdentifier& CurveId, int32 Flags, bool bShouldTransact = true) = 0;

	/**
	* Replace the keys for the transform curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the transform curve for which the keys are to be set
	* @param	TransformValues		Transform Values with which the existing values are to be replaced
	* @param	TimeKeys			Time Keys with which the existing keys are to be replaced
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the transform curve keys were succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetTransformCurveKeys(const FAnimationCurveIdentifier& CurveId, const TArray<FTransform>& TransformValues, const TArray<float>& TimeKeys, bool bShouldTransact = true) = 0;
		
	/**
	* Sets a single key for the transform curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	* In case a key for any of the individual transform channel curves already exists the value is replaced.
	*
	* @param	CurveId			    Identifier for the transform curve for which the key is to be set
	* @param	Time				Time of the key to be set
	* @param	Value				Value of the key to be set
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the transform curve key was succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetTransformCurveKey(const FAnimationCurveIdentifier& CurveId, float Time, const FTransform& Value, bool bShouldTransact = true) = 0;

	/**
	* Removes a single key for the transform curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the transform curve for which the key is to be removed
	* @param	Time				Time of the key to be removed
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the transform curve key was succesfully removed
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool RemoveTransformCurveKey(const FAnimationCurveIdentifier& CurveId, float Time, bool bShouldTransact = true) = 0;

	/**
	* Renames the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveRenamed notify if successful.
	*
	* @param	CurveToRenameId		Identifier for the curve to be renamed
	* @param	NewCurveId			Time of the key to be removed
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve was succesfully renamed
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool RenameCurve(const FAnimationCurveIdentifier& CurveToRenameId, const FAnimationCurveIdentifier& NewCurveId, bool bShouldTransact = true) = 0;

	/**
	* Changes the color of the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveRenamed notify if successful.
	* Currently changing curve colors is only supported for float curves.
	*
	* @param	CurveId				Identifier of the curve to change the color for
	* @param	Color				Color to which the curve is to be set
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve color was succesfully changed
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetCurveColor(const FAnimationCurveIdentifier& CurveId, FLinearColor Color, bool bShouldTransact = true) = 0;
	
	/**
	* Scales the curve with provided identifier. Broadcasts a EAnimDataModelNotifyType::CurveScaled notify if successful.
	*
	* @param	CurveId				Identifier of the curve to scale
	* @param	Origin				Time to use as the origin when scaling the curve
	* @param	Factor				Factor with which the curve is supposed to be scaled
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not scaling the curve was succesful
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool ScaleCurve(const FAnimationCurveIdentifier& CurveId, float Origin, float Factor, bool bShouldTransact = true) = 0;

	/**
	* Sets a single key for the curve with provided identifier and name. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	* In case a key for the provided key time already exists the key is replaced.
	*
	* @param	CurveId			    Identifier for the curve for which the key is to be set
	* @param	Key					Key to be set
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve key was succesfully set
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetCurveKey(const FAnimationCurveIdentifier& CurveId, const FRichCurveKey& Key, bool bShouldTransact = true) = 0;
	
	/**
	* Remove a single key from the curve with provided identifier and name. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the curve for which the key is to be removed
	* @param	Time				Time of the key to be removed
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not the curve key was succesfully removed
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool RemoveCurveKey(const FAnimationCurveIdentifier& CurveId, float Time, bool bShouldTransact = true) = 0;

	/**
	* Replace the keys for the curve with provided identifier and name. Broadcasts a EAnimDataModelNotifyType::CurveChanged notify if successful.
	*
	* @param	CurveId			    Identifier for the curve for which the keys are to be replaced
	* @param	CurveKeys			Keys with which the existing keys are to be replaced
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*
	* @return	Whether or not replacing curve keys was succesful
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual bool SetCurveKeys(const FAnimationCurveIdentifier& CurveId, const TArray<FRichCurveKey>& CurveKeys, bool bShouldTransact = true) = 0;

	/**
	* Updates the display name values for any stored curve, with the names being retrieved from the provided skeleton. Broadcasts a EAnimDataModelNotifyType::CurveRenamed for each to-be-updated curve name, wrapped within BracketOpened/BracketClosed notifies.
	*
	* @param	Skeleton			Skeleton to retrieve the display name values from
	* @param	SupportedCurveType	Curve type for which the names should be updated
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual void UpdateCurveNamesFromSkeleton(const USkeleton* Skeleton, ERawCurveTrackTypes SupportedCurveType, bool bShouldTransact = true) = 0;

	/**
	* Updates the curve names with the provided skeleton, if a display name is not found it will be added thus modifying the skeleton. Broadcasts a EAnimDataModelNotifyType::CurveRenamed for each curve name for which the UID was different or if it was added as a new smartname, wrapped within BracketOpened/BracketClosed notifies.
	*
	* @param	Skeleton			Skeleton to retrieve the display name values from
	* @param	SupportedCurveType	Curve type for which the names should be updated
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	UFUNCTION(BlueprintCallable, Category = CurveData)
	virtual void FindOrAddCurveNamesOnSkeleton(USkeleton* Skeleton, ERawCurveTrackTypes SupportedCurveType, bool bShouldTransact = true) = 0;

	/**
	* Removes any bone track for which the name was not found in the provided skeleton. Broadcasts a EAnimDataModelNotifyType::TrackRemoved for each track which was not found in the skeleton, wrapped within BracketOpened/BracketClosed notifies.
	*
	* @param	Skeleton			Skeleton to retrieve the display name values from
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	virtual bool RemoveBoneTracksMissingFromSkeleton(const USkeleton* Skeleton, bool bShouldTransact = true) = 0;

	/**
	* Broadcast a EAnimDataModelNotifyType::Populated notify.
	*/
	virtual void NotifyPopulated() = 0;	

	/**
	* Resets all data stored in the model, broadcasts a EAnimDataModelNotifyType::Reset and wraps all actions within BracketOpened/BracketClosed notifies.
	*	- Bone tracks, broadcasts a EAnimDataModelNotifyType::TrackRemoved for each
	*	- Curves, broadcasts a EAnimDataModelNotifyType::CurveRemoves for each
	*	- Play length to one frame at 30fps, broadcasts a EAnimDataModelNotifyType::PlayLengthChanged
	*	- Frame rate to 30fps, broadcasts a EAnimDataModelNotifyType::FrameRateChanged
	*
	* @param	bShouldTransact		Whether or not any undo-redo changes should be generated
	*/
	virtual void ResetModel(bool bShouldTransact = true) = 0;

protected:
	/** Functionality used by FOpenBracketAction and FCloseBracketAction to broadcast their equivalent notifies without actually opening a bracket. */
	virtual void NotifyBracketOpen() = 0;
	virtual void NotifyBracketClosed() = 0;

private:
	friend class FAnimDataControllerTestBase;
	friend UE::Anim::FOpenBracketAction;
	friend UE::Anim::FCloseBracketAction;
};
